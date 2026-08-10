use crate::{
    application::{StartExecutionRequest, TransitionWorkItemRequest},
    desktop_execution_runtime_support::{ExecutionRuntimeError, lock, timestamp},
    domain::{
        BoardSupervisionMode, EvidenceResult, SupervisionAction, SupervisionPolicyResult,
        TicketEffect, TicketEffectAction, TicketEffectOutcome, WorkItemId, WorkItemState,
    },
};

use super::{ExecutionRuntime, supervision::format_implementation_brief};

impl ExecutionRuntime {
    pub(super) fn apply_ticket_effect_under_gate(
        &self,
        effect: TicketEffect,
        automatically: bool,
    ) -> Result<TicketEffect, ExecutionRuntimeError> {
        if automatically && !self.autonomous_ticket_effect_allowed(&effect)? {
            return self.record_ticket_effect_outcome(
                effect,
                SupervisionPolicyResult::Denied,
                TicketEffectOutcome::Denied,
            );
        }
        let work_item = lock(&self.service, "board service")?
            .ticket_effect_work_item(&effect.work_item_id)
            .map_err(ExecutionRuntimeError::TicketEffect)?;
        if work_item.last_event_sequence != effect.expected_work_item_sequence {
            return self.record_ticket_effect_outcome(
                effect,
                SupervisionPolicyResult::NotRequired,
                TicketEffectOutcome::Stale,
            );
        }
        match effect.action {
            TicketEffectAction::RefineSpecification => {
                lock(&self.service, "board service")?
                    .refine_work_item_details(&work_item, &effect, timestamp())
                    .map_err(ExecutionRuntimeError::TicketEffect)?;
                self.record_ticket_effect_outcome(
                    effect,
                    SupervisionPolicyResult::NotRequired,
                    TicketEffectOutcome::Applied,
                )
            }
            TicketEffectAction::GiveWorkerGuidance | TicketEffectAction::ExplainEvidence => self
                .record_ticket_effect_outcome(
                    effect,
                    SupervisionPolicyResult::NotRequired,
                    TicketEffectOutcome::Applied,
                ),
            TicketEffectAction::PrepareStart => self.start_ticket_worker(effect, &work_item),
            TicketEffectAction::PrepareRestart => self.return_to_ready(
                effect,
                &work_item,
                matches!(
                    work_item.work_item.state,
                    WorkItemState::Failed | WorkItemState::Blocked
                ),
                "A reviewed task-AI restart preparation was applied.",
            ),
            TicketEffectAction::ReturnForCorrection => self.return_to_ready(
                effect,
                &work_item,
                work_item.work_item.state == WorkItemState::Review
                    && self.ticket_has_failed_evidence(&work_item.work_item.id)?,
                "A reviewed task-AI correction request was applied.",
            ),
            TicketEffectAction::RecoverInterrupted => self.return_to_ready(
                effect,
                &work_item,
                work_item.work_item.state == WorkItemState::Interrupted,
                "A reviewed task-AI interruption recovery was applied.",
            ),
        }
    }

    fn autonomous_ticket_effect_allowed(
        &self,
        effect: &TicketEffect,
    ) -> Result<bool, ExecutionRuntimeError> {
        if effect.authority_mode != BoardSupervisionMode::Autonomous {
            return Ok(false);
        }
        let supervision = lock(&self.service, "board service")?
            .board_supervision(&effect.board_id.0)
            .map_err(ExecutionRuntimeError::Supervision)?;
        let Some(supervision) = supervision else {
            return Ok(false);
        };
        if supervision.mode != BoardSupervisionMode::Autonomous
            || Some(supervision.revision) != effect.supervision_revision
        {
            return Ok(false);
        }
        let permitted_action = match effect.action {
            TicketEffectAction::GiveWorkerGuidance | TicketEffectAction::ExplainEvidence => {
                return Ok(true);
            }
            TicketEffectAction::PrepareStart => SupervisionAction::StartWork,
            TicketEffectAction::PrepareRestart | TicketEffectAction::RecoverInterrupted => {
                SupervisionAction::RetryWork
            }
            TicketEffectAction::ReturnForCorrection => SupervisionAction::ReturnForCorrection,
            TicketEffectAction::RefineSpecification => return Ok(false),
        };
        Ok(supervision.permitted_actions.contains(&permitted_action))
    }

    fn start_ticket_worker(
        &self,
        effect: TicketEffect,
        work_item: &crate::domain::MaterializedWorkItem,
    ) -> Result<TicketEffect, ExecutionRuntimeError> {
        if work_item.work_item.state != WorkItemState::Ready {
            return self.record_ticket_effect_outcome(
                effect,
                SupervisionPolicyResult::NotRequired,
                TicketEffectOutcome::Denied,
            );
        }
        let request = StartExecutionRequest {
            execution_id: format!("ticket-effect-{}", effect.id.0),
            work_item_id: effect.work_item_id.0.clone(),
            agent_profile_name: self.ticket_worker_name_for_effect(work_item)?,
            task_brief: self.ticket_worker_brief(work_item)?,
            execution_role: crate::domain::ExecutionRole::Implementation,
        };
        match self.start(request) {
            Ok(_) => self.record_ticket_effect_outcome(
                effect,
                SupervisionPolicyResult::Allowed,
                TicketEffectOutcome::Applied,
            ),
            Err(ExecutionRuntimeError::PolicyDenied { .. }) => self.record_ticket_effect_outcome(
                effect,
                SupervisionPolicyResult::Denied,
                TicketEffectOutcome::Denied,
            ),
            Err(ExecutionRuntimeError::WorkItemNotReady { .. }) => self
                .record_ticket_effect_outcome(
                    effect,
                    SupervisionPolicyResult::NotRequired,
                    TicketEffectOutcome::Stale,
                ),
            Err(error) => {
                self.record_ticket_effect_outcome(
                    effect,
                    SupervisionPolicyResult::NotRequired,
                    TicketEffectOutcome::Denied,
                )?;
                Err(error)
            }
        }
    }

    fn return_to_ready(
        &self,
        effect: TicketEffect,
        work_item: &crate::domain::MaterializedWorkItem,
        is_allowed: bool,
        reason: &str,
    ) -> Result<TicketEffect, ExecutionRuntimeError> {
        if !is_allowed {
            return self.record_ticket_effect_outcome(
                effect,
                SupervisionPolicyResult::NotRequired,
                TicketEffectOutcome::Denied,
            );
        }
        lock(&self.service, "board service")?
            .transition_work_item(TransitionWorkItemRequest {
                event_id: format!("ticket-effect-{}", effect.id.0),
                work_item_id: work_item.work_item.id.0.clone(),
                next_state: WorkItemState::Ready,
                evidence: None,
                reason: reason.to_owned(),
                recorded_at: timestamp(),
            })
            .map_err(ExecutionRuntimeError::Board)?;
        self.record_ticket_effect_outcome(
            effect,
            SupervisionPolicyResult::NotRequired,
            TicketEffectOutcome::Applied,
        )
    }

    fn ticket_has_failed_evidence(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<bool, ExecutionRuntimeError> {
        Ok(lock(&self.service, "board service")?
            .ticket_effect_evidence(work_item_id)
            .map_err(ExecutionRuntimeError::TicketEffect)?
            .iter()
            .any(|evidence| evidence.result == EvidenceResult::Failed))
    }

    fn ticket_worker_name_for_effect(
        &self,
        work_item: &crate::domain::MaterializedWorkItem,
    ) -> Result<String, ExecutionRuntimeError> {
        if let Some(profile_name) = &work_item.work_item.assigned_agent_profile_name {
            return Ok(profile_name.clone());
        }
        let service = lock(&self.service, "board service")?;
        let settings = service
            .project_agent_settings_for_board(&work_item.work_item.board_id.0)
            .map_err(ExecutionRuntimeError::ProjectAgentSettings)?;
        settings
            .and_then(|record| record.ticket_worker)
            .map(|worker| worker.agent_profile_name)
            .ok_or_else(|| ExecutionRuntimeError::TicketWorkerNotConfigured {
                work_item_id: work_item.work_item.id.clone(),
            })
    }

    fn ticket_worker_brief(
        &self,
        work_item: &crate::domain::MaterializedWorkItem,
    ) -> Result<String, ExecutionRuntimeError> {
        let guidance = lock(&self.service, "board service")?
            .applied_worker_guidance(&work_item.work_item.id)
            .map_err(ExecutionRuntimeError::TicketEffect)?;
        let brief = format_implementation_brief(&work_item.work_item);
        Ok(guidance.map_or(brief.clone(), |guidance| {
            format!("{brief}\n\nWorker guidance:\n{guidance}")
        }))
    }
}
