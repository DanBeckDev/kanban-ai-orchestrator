use std::{error::Error, fmt};

use crate::domain::{
    Evidence, MaterializedWorkItem, RefineWorkItemDetailsCommand, TicketEffect, TicketEffectId,
    WorkItemEventId, WorkItemId,
};

use super::{BoardRepository, BoardService, BoardServiceError};

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    pub(crate) fn record_ticket_effect(
        &mut self,
        effect: TicketEffect,
    ) -> Result<TicketEffect, TicketEffectServiceError<Repository::Error>> {
        self.repository
            .record_ticket_effect(effect)
            .map_err(TicketEffectServiceError::Repository)
    }

    pub(crate) fn update_ticket_effect(
        &mut self,
        effect: TicketEffect,
    ) -> Result<TicketEffect, TicketEffectServiceError<Repository::Error>> {
        self.repository
            .update_ticket_effect(effect)
            .map_err(TicketEffectServiceError::Repository)
    }

    pub(crate) fn ticket_effect(
        &self,
        effect_id: &TicketEffectId,
    ) -> Result<TicketEffect, TicketEffectServiceError<Repository::Error>> {
        self.repository
            .ticket_effect(effect_id)
            .map_err(TicketEffectServiceError::Repository)?
            .ok_or_else(|| TicketEffectServiceError::NotFound {
                effect_id: effect_id.clone(),
            })
    }

    pub fn ticket_effects_for_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Vec<TicketEffect>, TicketEffectServiceError<Repository::Error>> {
        self.repository
            .ticket_effects_for_work_item(work_item_id)
            .map_err(TicketEffectServiceError::Repository)
    }

    pub(crate) fn applied_worker_guidance(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Option<String>, TicketEffectServiceError<Repository::Error>> {
        Ok(self
            .ticket_effects_for_work_item(work_item_id)?
            .into_iter()
            .find_map(|effect| {
                (effect.action == crate::domain::TicketEffectAction::GiveWorkerGuidance
                    && effect.outcome == crate::domain::TicketEffectOutcome::Applied)
                    .then_some(effect.proposal.worker_guidance)
                    .flatten()
            }))
    }

    pub(crate) fn ticket_effect_evidence(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Vec<Evidence>, TicketEffectServiceError<Repository::Error>> {
        self.repository
            .evidence_for_work_item(work_item_id)
            .map_err(TicketEffectServiceError::Repository)
    }

    pub(crate) fn refine_work_item_details(
        &mut self,
        work_item: &MaterializedWorkItem,
        effect: &TicketEffect,
        recorded_at: String,
    ) -> Result<(), TicketEffectServiceError<Repository::Error>> {
        let title = effect.proposal.title.clone().ok_or_else(|| {
            TicketEffectServiceError::InvalidRefinement {
                effect_id: effect.id.clone(),
            }
        })?;
        let description = effect.proposal.description.clone().ok_or_else(|| {
            TicketEffectServiceError::InvalidRefinement {
                effect_id: effect.id.clone(),
            }
        })?;
        self.repository
            .refine_work_item_details(RefineWorkItemDetailsCommand {
                event_id: WorkItemEventId::from(format!("ticket-effect-{}", effect.id.0).as_str()),
                work_item_id: work_item.work_item.id.clone(),
                title,
                description,
                acceptance_criteria: effect.proposal.acceptance_criteria.clone(),
                expected_work_item_sequence: effect.expected_work_item_sequence,
                reason: "A reviewed task-AI refinement was applied.".to_owned(),
                recorded_at,
            })
            .map_err(TicketEffectServiceError::Repository)?;
        Ok(())
    }

    pub(crate) fn ticket_effect_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<MaterializedWorkItem, TicketEffectServiceError<Repository::Error>> {
        self.work_item(work_item_id)
            .map_err(TicketEffectServiceError::Board)
    }
}

#[derive(Debug)]
pub enum TicketEffectServiceError<RepositoryError> {
    Repository(RepositoryError),
    Board(BoardServiceError<RepositoryError>),
    NotFound { effect_id: TicketEffectId },
    InvalidRefinement { effect_id: TicketEffectId },
}

impl<RepositoryError> fmt::Display for TicketEffectServiceError<RepositoryError>
where
    RepositoryError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(error) => write!(formatter, "ticket-effect storage error: {error}"),
            Self::Board(error) => write!(formatter, "ticket-effect board error: {error}"),
            Self::NotFound { effect_id } => {
                write!(formatter, "ticket effect {} was not found", effect_id.0)
            }
            Self::InvalidRefinement { effect_id } => {
                write!(
                    formatter,
                    "ticket effect {} has no complete refinement",
                    effect_id.0
                )
            }
        }
    }
}

impl<RepositoryError> Error for TicketEffectServiceError<RepositoryError>
where
    RepositoryError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Repository(error) => Some(error),
            Self::Board(error) => Some(error),
            Self::NotFound { .. } | Self::InvalidRefinement { .. } => None,
        }
    }
}
