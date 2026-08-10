use uuid::Uuid;

use std::path::Path;

use crate::{
    application::{BoardSnapshot, TransitionWorkItemRequest},
    desktop_execution_runtime_support::{ExecutionRuntimeError, lock, timestamp},
    domain::{
        BoardId, BoardSupervision, BoardSupervisionMode, SchemaMetadata, SupervisionAction,
        SupervisionDecision, SupervisionDecisionId, SupervisionDecisionOutcome,
        SupervisionPolicyResult, WorkItemState,
    },
    orchestration::ProcessBoardSupervisor,
};

use super::{
    ExecutionRuntime,
    supervision_selection::{SupervisionCandidate, candidates, organiser_input},
};

const MAX_SUPERVISION_ACTIONS_PER_PASS: usize = 100;

impl ExecutionRuntime {
    pub(crate) fn coordinate_board(
        &self,
        board_id: &str,
    ) -> Result<BoardSnapshot, ExecutionRuntimeError> {
        require_value(board_id, "board id")?;
        let supervision = self.board_supervision(board_id)?;
        if supervision.mode == BoardSupervisionMode::Manual {
            return self.record_manual_recommendation(&supervision);
        }
        self.reconcile_pending_decisions(&supervision)?;
        self.run_autonomous_supervision(&supervision)
    }

    fn board_supervision(&self, board_id: &str) -> Result<BoardSupervision, ExecutionRuntimeError> {
        lock(&self.service, "board service")?
            .board_supervision(board_id)
            .map_err(ExecutionRuntimeError::Supervision)?
            .ok_or_else(|| ExecutionRuntimeError::SupervisionNotConfigured {
                board_id: board_id.to_owned(),
            })
    }

    fn record_manual_recommendation(
        &self,
        supervision: &BoardSupervision,
    ) -> Result<BoardSnapshot, ExecutionRuntimeError> {
        let snapshot = self.snapshot(&supervision.board_id)?;
        let Some(candidate) = self.organiser_candidate(supervision, &snapshot)? else {
            return Ok(snapshot);
        };
        let decision = self.record_decision(supervision, &candidate)?;
        if decision.outcome == SupervisionDecisionOutcome::Pending {
            self.resolve_decision(
                decision,
                SupervisionPolicyResult::NotRequired,
                SupervisionDecisionOutcome::RecommendedForApproval,
            )?;
        }
        self.snapshot(&supervision.board_id)
    }

    fn run_autonomous_supervision(
        &self,
        supervision: &BoardSupervision,
    ) -> Result<BoardSnapshot, ExecutionRuntimeError> {
        for _ in 0..MAX_SUPERVISION_ACTIONS_PER_PASS {
            if !self.autonomy_is_active(supervision)? {
                return self.snapshot(&supervision.board_id);
            }
            let snapshot = self.snapshot(&supervision.board_id)?;
            let Some(candidate) = self.organiser_candidate(supervision, &snapshot)? else {
                return Ok(snapshot);
            };
            if candidate.action == SupervisionAction::StartWork
                && active_execution_count(&snapshot) >= supervision.limits.max_parallel_work_items
            {
                self.record_capacity_denial(supervision, &candidate)?;
                return self.snapshot(&supervision.board_id);
            }
            if self.apply_candidate(supervision, candidate)? {
                return self.snapshot(&supervision.board_id);
            }
        }
        Err(ExecutionRuntimeError::SupervisionLoopLimit {
            board_id: supervision.board_id.0.clone(),
        })
    }

    fn apply_candidate(
        &self,
        supervision: &BoardSupervision,
        candidate: SupervisionCandidate,
    ) -> Result<bool, ExecutionRuntimeError> {
        let _supervision_gate = lock(&self.supervision_gate, "board supervision")?;
        if !self.autonomy_is_active(supervision)? {
            return Ok(false);
        }
        let decision = self.record_decision(supervision, &candidate)?;
        if decision.outcome != SupervisionDecisionOutcome::Pending {
            return Ok(decision.action == SupervisionAction::StartWork);
        }
        if !self.is_current(&candidate)? {
            self.resolve_decision(
                decision,
                SupervisionPolicyResult::NotRequired,
                SupervisionDecisionOutcome::Stale,
            )?;
            return Ok(false);
        }
        match candidate.action {
            SupervisionAction::PrepareWork => self.transition(
                &candidate,
                WorkItemState::Planned,
                "Kanban prepared this task for dependency scheduling.",
            )?,
            SupervisionAction::MakeWorkReady => self.transition(
                &candidate,
                WorkItemState::Ready,
                "Kanban found that all required upstream work is complete.",
            )?,
            SupervisionAction::RetryWork => self.transition(
                &candidate,
                WorkItemState::Ready,
                "Kanban returned this recoverable task to ready with its existing evidence retained.",
            )?,
            SupervisionAction::ReturnForCorrection => self.transition(
                &candidate,
                WorkItemState::Ready,
                "Kanban returned this task to ready after recorded review evidence required correction.",
            )?,
            SupervisionAction::StartWork => return self.start_candidate(decision, candidate),
        }
        self.resolve_decision(
            decision,
            SupervisionPolicyResult::NotRequired,
            SupervisionDecisionOutcome::Executed,
        )?;
        Ok(false)
    }

    fn start_candidate(
        &self,
        decision: SupervisionDecision,
        candidate: SupervisionCandidate,
    ) -> Result<bool, ExecutionRuntimeError> {
        let request = crate::application::StartExecutionRequest {
            execution_id: format!("orchestrator-{}-{}", candidate.work_item_id, Uuid::new_v4()),
            work_item_id: candidate.work_item_id,
            agent_profile_name: self.ticket_worker_name(&decision.board_id)?,
            task_brief: self
                .implementation_brief(&decision.board_id, decision.work_item_id.as_ref())?,
            execution_role: crate::domain::ExecutionRole::Implementation,
        };
        match self.start(request) {
            Ok(_) => {
                self.resolve_decision(
                    decision,
                    SupervisionPolicyResult::Allowed,
                    SupervisionDecisionOutcome::Executed,
                )?;
                Ok(true)
            }
            Err(ExecutionRuntimeError::PolicyDenied { .. }) => {
                self.resolve_decision(
                    decision,
                    SupervisionPolicyResult::Denied,
                    SupervisionDecisionOutcome::Denied,
                )?;
                Ok(true)
            }
            Err(ExecutionRuntimeError::WorkItemNotReady { .. }) => {
                self.resolve_decision(
                    decision,
                    SupervisionPolicyResult::NotRequired,
                    SupervisionDecisionOutcome::Stale,
                )?;
                Ok(false)
            }
            Err(error) => {
                self.resolve_decision(
                    decision,
                    SupervisionPolicyResult::NotRequired,
                    SupervisionDecisionOutcome::Denied,
                )?;
                Err(error)
            }
        }
    }

    fn ticket_worker_name(&self, board_id: &BoardId) -> Result<String, ExecutionRuntimeError> {
        self.board_supervision(&board_id.0)
            .map(|supervision| supervision.ticket_worker.agent_profile_name)
    }

    fn organiser_candidate(
        &self,
        supervision: &BoardSupervision,
        snapshot: &BoardSnapshot,
    ) -> Result<Option<SupervisionCandidate>, ExecutionRuntimeError> {
        let candidates = candidates(snapshot, supervision);
        if candidates.is_empty() {
            return Ok(None);
        }
        let input = organiser_input(snapshot, &candidates)?;
        let context = lock(&self.service, "board service")?
            .planner_context(
                &supervision.board_id.0,
                &supervision.organiser.planner_profile_name,
            )
            .map_err(ExecutionRuntimeError::Planner)?;
        let recommendation = ProcessBoardSupervisor::recommend(
            &context.profile,
            Path::new(&context.repository_path),
            &input,
        )
        .map_err(ExecutionRuntimeError::Supervisor)?;
        let Some(mut candidate) = candidates.into_iter().find(|candidate| {
            candidate.action == recommendation.action
                && candidate.work_item_id == recommendation.work_item_id
        }) else {
            return Err(ExecutionRuntimeError::SupervisionDecisionInvalid {
                reason: "organiser selected a candidate that the daemon did not offer".to_owned(),
            });
        };
        candidate.recommendation = recommendation.recommendation;
        candidate.rationale = recommendation.rationale;
        Ok(Some(candidate))
    }

    fn implementation_brief(
        &self,
        board_id: &BoardId,
        work_item_id: Option<&crate::domain::WorkItemId>,
    ) -> Result<String, ExecutionRuntimeError> {
        let work_item_id =
            work_item_id.ok_or_else(|| ExecutionRuntimeError::SupervisionDecisionInvalid {
                reason: "a start decision needs a work item".to_owned(),
            })?;
        let service = lock(&self.service, "board service")?;
        let work_item = service
            .work_item(work_item_id)
            .map_err(ExecutionRuntimeError::Board)?;
        if work_item.work_item.board_id != *board_id {
            return Err(ExecutionRuntimeError::SupervisionDecisionInvalid {
                reason: "a start decision cannot cross boards".to_owned(),
            });
        }
        Ok(format_implementation_brief(&work_item.work_item))
    }

    fn transition(
        &self,
        candidate: &SupervisionCandidate,
        next_state: WorkItemState,
        reason: &str,
    ) -> Result<(), ExecutionRuntimeError> {
        lock(&self.service, "board service")?
            .transition_work_item(TransitionWorkItemRequest {
                event_id: format!("orchestrator-{}-{}", next_state, Uuid::new_v4()),
                work_item_id: candidate.work_item_id.clone(),
                next_state,
                evidence: None,
                reason: reason.to_owned(),
                recorded_at: timestamp(),
            })
            .map(|_| ())
            .map_err(ExecutionRuntimeError::Board)
    }

    fn is_current(&self, candidate: &SupervisionCandidate) -> Result<bool, ExecutionRuntimeError> {
        let work_item = lock(&self.service, "board service")?
            .work_item(&crate::domain::WorkItemId::from(
                candidate.work_item_id.as_str(),
            ))
            .map_err(ExecutionRuntimeError::Board)?;
        Ok(work_item.last_event_sequence == candidate.expected_sequence)
    }

    fn record_decision(
        &self,
        supervision: &BoardSupervision,
        candidate: &SupervisionCandidate,
    ) -> Result<SupervisionDecision, ExecutionRuntimeError> {
        lock(&self.service, "board service")?
            .record_supervision_decision(SupervisionDecision {
                schema: SchemaMetadata::current(),
                id: SupervisionDecisionId::from(Uuid::new_v4().to_string().as_str()),
                board_id: supervision.board_id.clone(),
                work_item_id: Some(crate::domain::WorkItemId::from(
                    candidate.work_item_id.as_str(),
                )),
                organiser_profile_name: supervision.organiser.planner_profile_name.clone(),
                action: candidate.action,
                recommendation: candidate.recommendation.clone(),
                rationale: candidate.rationale.clone(),
                policy_result: SupervisionPolicyResult::NotRequired,
                outcome: SupervisionDecisionOutcome::Pending,
                idempotency_key: idempotency_key(supervision, candidate),
                expected_work_item_sequence: Some(candidate.expected_sequence),
                recorded_at: timestamp(),
                resolved_at: None,
            })
            .map_err(ExecutionRuntimeError::Supervision)
    }

    fn resolve_decision(
        &self,
        mut decision: SupervisionDecision,
        policy_result: SupervisionPolicyResult,
        outcome: SupervisionDecisionOutcome,
    ) -> Result<(), ExecutionRuntimeError> {
        decision.policy_result = policy_result;
        decision.outcome = outcome;
        decision.resolved_at = Some(timestamp());
        lock(&self.service, "board service")?
            .resolve_supervision_decision(decision)
            .map(|_| ())
            .map_err(ExecutionRuntimeError::Supervision)
    }

    fn record_capacity_denial(
        &self,
        supervision: &BoardSupervision,
        candidate: &SupervisionCandidate,
    ) -> Result<(), ExecutionRuntimeError> {
        let _supervision_gate = lock(&self.supervision_gate, "board supervision")?;
        if !self.autonomy_is_active(supervision)? {
            return Ok(());
        }
        let decision = self.record_decision(supervision, candidate)?;
        if decision.outcome == SupervisionDecisionOutcome::Pending {
            self.resolve_decision(
                decision,
                SupervisionPolicyResult::Denied,
                SupervisionDecisionOutcome::Denied,
            )?;
        }
        Ok(())
    }

    fn reconcile_pending_decisions(
        &self,
        supervision: &BoardSupervision,
    ) -> Result<(), ExecutionRuntimeError> {
        let decisions = lock(&self.service, "board service")?
            .supervision_decisions(&supervision.board_id)
            .map_err(ExecutionRuntimeError::Supervision)?;
        for decision in decisions
            .into_iter()
            .filter(|record| record.outcome == SupervisionDecisionOutcome::Pending)
        {
            self.resolve_decision(
                decision,
                SupervisionPolicyResult::NotRequired,
                SupervisionDecisionOutcome::Recovered,
            )?;
        }
        Ok(())
    }

    fn autonomy_is_active(
        &self,
        expected: &BoardSupervision,
    ) -> Result<bool, ExecutionRuntimeError> {
        let current = self.board_supervision(&expected.board_id.0)?;
        Ok(current.mode == BoardSupervisionMode::Autonomous
            && current.revision == expected.revision)
    }

    fn snapshot(&self, board_id: &BoardId) -> Result<BoardSnapshot, ExecutionRuntimeError> {
        lock(&self.service, "board service")?
            .snapshot(board_id)
            .map_err(ExecutionRuntimeError::Board)
    }
}

fn active_execution_count(snapshot: &BoardSnapshot) -> u32 {
    snapshot
        .executions
        .iter()
        .filter(|execution| {
            matches!(
                execution.status,
                crate::domain::ExecutionStatus::Pending
                    | crate::domain::ExecutionStatus::Running
                    | crate::domain::ExecutionStatus::AwaitingInput
            )
        })
        .count() as u32
}

fn idempotency_key(supervision: &BoardSupervision, candidate: &SupervisionCandidate) -> String {
    format!(
        "{}:{}:{}:{:?}",
        supervision.revision, candidate.work_item_id, candidate.expected_sequence, candidate.action
    )
}

fn format_implementation_brief(work_item: &crate::domain::WorkItem) -> String {
    let criteria = work_item
        .acceptance_criteria
        .iter()
        .map(|criterion| format!("- {criterion}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Implement {}.\n\n{}\n\nAcceptance criteria:\n{}",
        work_item.title, work_item.description, criteria
    )
}

fn require_value(value: &str, field: &'static str) -> Result<(), ExecutionRuntimeError> {
    if value.trim().is_empty() {
        Err(ExecutionRuntimeError::MissingRequiredField { field })
    } else {
        Ok(())
    }
}
