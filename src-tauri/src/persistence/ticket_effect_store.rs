use rusqlite::{OptionalExtension, params};

use crate::domain::{TicketEffect, TicketEffectId, TicketEffectOutcome, WorkItemId};

use super::{EventStoreError, SqliteEventStore};

impl SqliteEventStore {
    pub fn record_ticket_effect(
        &mut self,
        effect: TicketEffect,
    ) -> Result<TicketEffect, EventStoreError> {
        if let Some(recorded) = self.ticket_effect(&effect.id)? {
            return matching_effect(recorded, effect);
        }
        if let Some(recorded) =
            self.ticket_effect_by_key(&effect.board_id, &effect.idempotency_key)?
        {
            return Ok(recorded);
        }
        self.connection.execute(
            "INSERT INTO ticket_effects (
                effect_id, board_id, work_item_id, idempotency_key, recorded_at, effect_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                effect.id.0,
                effect.board_id.0,
                effect.work_item_id.0,
                effect.idempotency_key,
                effect.recorded_at,
                serde_json::to_string(&effect)?,
            ],
        )?;
        Ok(effect)
    }

    pub fn update_ticket_effect(
        &mut self,
        effect: TicketEffect,
    ) -> Result<TicketEffect, EventStoreError> {
        let recorded = self.ticket_effect(&effect.id)?.ok_or_else(|| {
            EventStoreError::TicketEffectNotFound {
                effect_id: effect.id.clone(),
            }
        })?;
        if !valid_outcome_update(&recorded, &effect) {
            return Err(EventStoreError::TicketEffectInvalidOutcomeTransition {
                effect_id: effect.id,
            });
        }
        self.connection.execute(
            "UPDATE ticket_effects SET effect_json = ?2 WHERE effect_id = ?1",
            params![effect.id.0, serde_json::to_string(&effect)?],
        )?;
        Ok(effect)
    }

    pub fn ticket_effect(
        &self,
        effect_id: &TicketEffectId,
    ) -> Result<Option<TicketEffect>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT effect_json FROM ticket_effects WHERE effect_id = ?1",
                [effect_id.0.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|stored| Ok(serde_json::from_str(&stored)?))
            .transpose()
    }

    pub fn ticket_effects_for_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Vec<TicketEffect>, EventStoreError> {
        let mut statement = self.connection.prepare(
            "SELECT effect_json
             FROM ticket_effects
             WHERE work_item_id = ?1
             ORDER BY recorded_at DESC, effect_id DESC
             LIMIT 50",
        )?;
        let rows = statement.query_map([work_item_id.0.as_str()], |row| row.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }

    fn ticket_effect_by_key(
        &self,
        board_id: &crate::domain::BoardId,
        idempotency_key: &str,
    ) -> Result<Option<TicketEffect>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT effect_json FROM ticket_effects
                 WHERE board_id = ?1 AND idempotency_key = ?2",
                params![board_id.0, idempotency_key],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|stored| Ok(serde_json::from_str(&stored)?))
            .transpose()
    }
}

fn matching_effect(
    recorded: TicketEffect,
    requested: TicketEffect,
) -> Result<TicketEffect, EventStoreError> {
    if recorded == requested {
        Ok(recorded)
    } else {
        Err(EventStoreError::TicketEffectConflict {
            effect_id: requested.id,
        })
    }
}

fn valid_outcome_update(recorded: &TicketEffect, requested: &TicketEffect) -> bool {
    if recorded.id != requested.id
        || recorded.board_id != requested.board_id
        || recorded.work_item_id != requested.work_item_id
        || recorded.organiser_profile_name != requested.organiser_profile_name
        || recorded.action != requested.action
        || recorded.prompt_summary != requested.prompt_summary
        || recorded.recommendation != requested.recommendation
        || recorded.rationale != requested.rationale
        || recorded.proposal != requested.proposal
        || recorded.authority_mode != requested.authority_mode
        || recorded.supervision_revision != requested.supervision_revision
        || recorded.idempotency_key != requested.idempotency_key
        || recorded.expected_work_item_sequence != requested.expected_work_item_sequence
        || recorded.recorded_at != requested.recorded_at
    {
        return false;
    }
    matches!(
        (recorded.outcome, requested.outcome),
        (
            TicketEffectOutcome::Pending,
            TicketEffectOutcome::AwaitingApproval
        ) | (TicketEffectOutcome::Pending, TicketEffectOutcome::Applied)
            | (TicketEffectOutcome::Pending, TicketEffectOutcome::Denied)
            | (TicketEffectOutcome::Pending, TicketEffectOutcome::Stale)
            | (TicketEffectOutcome::Pending, TicketEffectOutcome::Recovered)
            | (
                TicketEffectOutcome::AwaitingApproval,
                TicketEffectOutcome::Applied
            )
            | (
                TicketEffectOutcome::AwaitingApproval,
                TicketEffectOutcome::Rejected
            )
            | (
                TicketEffectOutcome::AwaitingApproval,
                TicketEffectOutcome::Cancelled
            )
            | (
                TicketEffectOutcome::AwaitingApproval,
                TicketEffectOutcome::Denied
            )
            | (
                TicketEffectOutcome::AwaitingApproval,
                TicketEffectOutcome::Stale
            )
    )
}
