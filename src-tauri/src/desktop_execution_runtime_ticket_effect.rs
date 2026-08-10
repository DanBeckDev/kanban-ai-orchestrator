use std::path::Path;

use crate::{
    application::{BoardSnapshot, ResolveTicketEffectRequest, TicketEffectPromptRequest},
    desktop_execution_runtime_support::{ExecutionRuntimeError, lock, timestamp},
    domain::{
        BoardSupervision, BoardSupervisionMode, SchemaMetadata, SupervisionPolicyResult,
        TicketEffect, TicketEffectId, TicketEffectOutcome, TicketEffectResolution, WorkItemId,
    },
    orchestration::{
        ProcessTicketEffectAdvisor, TicketEffectEvidence, TicketEffectInput, TicketEffectTask,
    },
};

use super::ExecutionRuntime;

pub(super) struct TicketEffectContext {
    pub(super) work_item: crate::domain::MaterializedWorkItem,
    pub(super) supervision: Option<BoardSupervision>,
    pub(super) organiser_profile_name: String,
    pub(super) organiser_model: crate::domain::AgentModelPreference,
    pub(super) organiser_effort: crate::domain::AgentEffort,
    pub(super) profile: crate::orchestration::PlannerProfile,
    pub(super) repository_path: String,
    pub(super) evidence: Vec<crate::domain::Evidence>,
}

impl ExecutionRuntime {
    pub(crate) fn request_ticket_effect(
        &self,
        request: TicketEffectPromptRequest,
    ) -> Result<TicketEffect, ExecutionRuntimeError> {
        require_ticket_request(&request)?;
        let _effect_gate = lock(&self.ticket_effect_gate, "ticket effect")?;
        if let Some(effect) = self.existing_ticket_effect(&request)? {
            return self.recover_pending_ticket_effect(effect);
        }
        let context = self.ticket_effect_context(&request.work_item_id)?;
        let input = ticket_effect_input(&request, &context)?;
        let recommendation = ProcessTicketEffectAdvisor::advise_with_preferences(
            &context.profile,
            Path::new(&context.repository_path),
            &input,
            &context.organiser_model,
            context.organiser_effort,
        )
        .map_err(ExecutionRuntimeError::TicketEffectAdvisor)?;
        let effect = prepared_effect(&request, &context, &input, recommendation);
        let recorded = lock(&self.service, "board service")?
            .record_ticket_effect(effect)
            .map_err(ExecutionRuntimeError::TicketEffect)?;
        if recorded.outcome != TicketEffectOutcome::Pending {
            return Ok(recorded);
        }
        if !self.ticket_effect_is_current(&recorded)? {
            return self.record_ticket_effect_outcome(
                recorded,
                SupervisionPolicyResult::NotRequired,
                TicketEffectOutcome::Stale,
            );
        }
        if recorded.authority_mode == BoardSupervisionMode::Autonomous {
            return self.apply_ticket_effect_under_gate(recorded, true);
        }
        if recorded.action.requires_user_decision_in_manual_mode() {
            return self.record_ticket_effect_outcome(
                recorded,
                SupervisionPolicyResult::NotRequired,
                TicketEffectOutcome::AwaitingApproval,
            );
        }
        self.apply_ticket_effect_under_gate(recorded, false)
    }

    pub(crate) fn resolve_ticket_effect(
        &self,
        request: ResolveTicketEffectRequest,
    ) -> Result<BoardSnapshot, ExecutionRuntimeError> {
        if request.effect_id.trim().is_empty() {
            return Err(ExecutionRuntimeError::MissingRequiredField {
                field: "ticket effect id",
            });
        }
        let _effect_gate = lock(&self.ticket_effect_gate, "ticket effect")?;
        let effect = lock(&self.service, "board service")?
            .ticket_effect(&TicketEffectId::from(request.effect_id.as_str()))
            .map_err(ExecutionRuntimeError::TicketEffect)?;
        if effect.outcome != TicketEffectOutcome::AwaitingApproval {
            return self.snapshot_for_ticket_effect(&effect);
        }
        match request.resolution {
            TicketEffectResolution::Apply => {
                self.apply_ticket_effect_under_gate(effect, false)?;
            }
            TicketEffectResolution::Reject => {
                self.record_ticket_effect_outcome(
                    effect,
                    SupervisionPolicyResult::NotRequired,
                    TicketEffectOutcome::Rejected,
                )?;
            }
            TicketEffectResolution::Cancel => {
                self.record_ticket_effect_outcome(
                    effect,
                    SupervisionPolicyResult::NotRequired,
                    TicketEffectOutcome::Cancelled,
                )?;
            }
        }
        self.snapshot_for_effect_id(&request.effect_id)
    }

    pub(crate) fn ticket_effects_for_work_item(
        &self,
        work_item_id: &str,
    ) -> Result<Vec<TicketEffect>, ExecutionRuntimeError> {
        let work_item_id = WorkItemId::from(work_item_id);
        self.recover_ticket_effects(&work_item_id)?;
        lock(&self.service, "board service")?
            .ticket_effects_for_work_item(&work_item_id)
            .map_err(ExecutionRuntimeError::TicketEffect)
    }

    pub(super) fn record_ticket_effect_outcome(
        &self,
        mut effect: TicketEffect,
        policy_result: SupervisionPolicyResult,
        outcome: TicketEffectOutcome,
    ) -> Result<TicketEffect, ExecutionRuntimeError> {
        effect.policy_result = policy_result;
        effect.outcome = outcome;
        effect.outcome_at = Some(timestamp());
        lock(&self.service, "board service")?
            .update_ticket_effect(effect)
            .map_err(ExecutionRuntimeError::TicketEffect)
    }

    pub(super) fn ticket_effect_is_current(
        &self,
        effect: &TicketEffect,
    ) -> Result<bool, ExecutionRuntimeError> {
        let work_item = lock(&self.service, "board service")?
            .ticket_effect_work_item(&effect.work_item_id)
            .map_err(ExecutionRuntimeError::TicketEffect)?;
        Ok(work_item.last_event_sequence == effect.expected_work_item_sequence)
    }

    fn ticket_effect_context(
        &self,
        work_item_id: &str,
    ) -> Result<TicketEffectContext, ExecutionRuntimeError> {
        let service = lock(&self.service, "board service")?;
        let work_item = service
            .ticket_effect_work_item(&WorkItemId::from(work_item_id))
            .map_err(ExecutionRuntimeError::TicketEffect)?;
        let board_id = work_item.work_item.board_id.clone();
        let supervision = service
            .board_supervision(&board_id.0)
            .map_err(ExecutionRuntimeError::Supervision)?;
        let settings = service
            .project_agent_settings_for_board(&board_id.0)
            .map_err(ExecutionRuntimeError::ProjectAgentSettings)?;
        let organiser = supervision
            .as_ref()
            .map(|record| record.organiser.clone())
            .or_else(|| settings.and_then(|record| record.organiser))
            .ok_or_else(|| ExecutionRuntimeError::OrganiserNotConfigured {
                board_id: board_id.0.clone(),
            })?;
        let planner = service
            .planner_context(&board_id.0, &organiser.planner_profile_name)
            .map_err(ExecutionRuntimeError::Planner)?;
        let evidence = service
            .ticket_effect_evidence(&work_item.work_item.id)
            .map_err(ExecutionRuntimeError::TicketEffect)?;
        Ok(TicketEffectContext {
            work_item,
            supervision,
            organiser_profile_name: organiser.planner_profile_name,
            organiser_model: organiser.model,
            organiser_effort: organiser.effort,
            profile: planner.profile,
            repository_path: planner.repository_path,
            evidence,
        })
    }

    fn existing_ticket_effect(
        &self,
        request: &TicketEffectPromptRequest,
    ) -> Result<Option<TicketEffect>, ExecutionRuntimeError> {
        Ok(lock(&self.service, "board service")?
            .ticket_effects_for_work_item(&WorkItemId::from(request.work_item_id.as_str()))
            .map_err(ExecutionRuntimeError::TicketEffect)?
            .into_iter()
            .find(|effect| effect.id.0 == request.request_id))
    }

    fn recover_pending_ticket_effect(
        &self,
        effect: TicketEffect,
    ) -> Result<TicketEffect, ExecutionRuntimeError> {
        if effect.outcome != TicketEffectOutcome::Pending {
            return Ok(effect);
        }
        let outcome = if effect.authority_mode == BoardSupervisionMode::Manual {
            TicketEffectOutcome::AwaitingApproval
        } else {
            TicketEffectOutcome::Recovered
        };
        self.record_ticket_effect_outcome(effect, SupervisionPolicyResult::NotRequired, outcome)
    }

    fn recover_ticket_effects(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<(), ExecutionRuntimeError> {
        let _effect_gate = lock(&self.ticket_effect_gate, "ticket effect")?;
        let effects = lock(&self.service, "board service")?
            .ticket_effects_for_work_item(work_item_id)
            .map_err(ExecutionRuntimeError::TicketEffect)?;
        for effect in effects
            .into_iter()
            .filter(|effect| effect.outcome == TicketEffectOutcome::Pending)
        {
            self.recover_pending_ticket_effect(effect)?;
        }
        Ok(())
    }

    fn snapshot_for_effect_id(
        &self,
        effect_id: &str,
    ) -> Result<BoardSnapshot, ExecutionRuntimeError> {
        let effect = lock(&self.service, "board service")?
            .ticket_effect(&TicketEffectId::from(effect_id))
            .map_err(ExecutionRuntimeError::TicketEffect)?;
        self.snapshot_for_ticket_effect(&effect)
    }

    fn snapshot_for_ticket_effect(
        &self,
        effect: &TicketEffect,
    ) -> Result<BoardSnapshot, ExecutionRuntimeError> {
        lock(&self.service, "board service")?
            .snapshot(&effect.board_id)
            .map_err(ExecutionRuntimeError::Board)
    }
}

fn ticket_effect_input(
    request: &TicketEffectPromptRequest,
    context: &TicketEffectContext,
) -> Result<TicketEffectInput, ExecutionRuntimeError> {
    TicketEffectInput::new(
        request.action,
        &request.prompt,
        TicketEffectTask {
            title: context.work_item.work_item.title.clone(),
            description: context.work_item.work_item.description.clone(),
            acceptance_criteria: context.work_item.work_item.acceptance_criteria.clone(),
            state: context.work_item.work_item.state.to_string(),
        },
        context
            .evidence
            .iter()
            .map(|evidence| TicketEffectEvidence {
                kind: format!("{:?}", evidence.kind),
                result: format!("{:?}", evidence.result),
                summary: evidence.summary.clone(),
            })
            .collect(),
    )
    .map_err(ExecutionRuntimeError::TicketEffectInput)
}

fn prepared_effect(
    request: &TicketEffectPromptRequest,
    context: &TicketEffectContext,
    input: &TicketEffectInput,
    recommendation: crate::orchestration::TicketEffectRecommendation,
) -> TicketEffect {
    let supervision = context.supervision.as_ref();
    TicketEffect {
        schema: SchemaMetadata::current(),
        id: TicketEffectId::from(request.request_id.as_str()),
        board_id: context.work_item.work_item.board_id.clone(),
        work_item_id: context.work_item.work_item.id.clone(),
        organiser_profile_name: context.organiser_profile_name.clone(),
        action: request.action,
        prompt_summary: input.prompt.clone(),
        recommendation: recommendation.recommendation,
        rationale: recommendation.rationale,
        proposal: recommendation.proposal,
        authority_mode: supervision
            .map(|record| record.mode)
            .unwrap_or(BoardSupervisionMode::Manual),
        supervision_revision: supervision.map(|record| record.revision),
        policy_result: SupervisionPolicyResult::NotRequired,
        outcome: TicketEffectOutcome::Pending,
        idempotency_key: request.request_id.clone(),
        expected_work_item_sequence: context.work_item.last_event_sequence,
        recorded_at: timestamp(),
        outcome_at: None,
    }
}

fn require_ticket_request(
    request: &TicketEffectPromptRequest,
) -> Result<(), ExecutionRuntimeError> {
    for (value, field) in [
        (&request.request_id, "ticket effect request id"),
        (&request.work_item_id, "work item id"),
        (&request.prompt, "ticket AI prompt"),
    ] {
        if value.trim().is_empty() {
            return Err(ExecutionRuntimeError::MissingRequiredField { field });
        }
    }
    Ok(())
}
