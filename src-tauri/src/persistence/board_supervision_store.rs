use rusqlite::{OptionalExtension, params};

use crate::domain::{BoardId, BoardSupervision, SupervisionDecision, SupervisionDecisionId};

use super::{EventStoreError, SqliteEventStore};

impl SqliteEventStore {
    pub fn save_board_supervision(
        &mut self,
        supervision: BoardSupervision,
    ) -> Result<BoardSupervision, EventStoreError> {
        let supervision_json = serde_json::to_string(&supervision)?;
        self.connection.execute(
            "INSERT INTO board_supervisions (board_id, supervision_json)
             VALUES (?1, ?2)
             ON CONFLICT(board_id) DO UPDATE SET supervision_json = excluded.supervision_json",
            params![supervision.board_id.0, supervision_json],
        )?;
        Ok(supervision)
    }

    pub fn board_supervision(
        &self,
        board_id: &BoardId,
    ) -> Result<Option<BoardSupervision>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT supervision_json FROM board_supervisions WHERE board_id = ?1",
                [board_id.0.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|stored| Ok(serde_json::from_str(&stored)?))
            .transpose()
    }

    pub fn record_supervision_decision(
        &mut self,
        decision: SupervisionDecision,
    ) -> Result<SupervisionDecision, EventStoreError> {
        if let Some(recorded) = self.supervision_decision(&decision.id)? {
            return matching_decision(recorded, decision);
        }
        if let Some(recorded) =
            self.supervision_decision_by_key(&decision.board_id, &decision.idempotency_key)?
        {
            return Ok(recorded);
        }
        let decision_json = serde_json::to_string(&decision)?;
        self.connection.execute(
            "INSERT INTO supervision_decisions (
                decision_id, board_id, work_item_id, idempotency_key, recorded_at, decision_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                decision.id.0,
                decision.board_id.0,
                decision.work_item_id.as_ref().map(|id| id.0.as_str()),
                decision.idempotency_key,
                decision.recorded_at,
                decision_json,
            ],
        )?;
        Ok(decision)
    }

    pub fn resolve_supervision_decision(
        &mut self,
        decision: SupervisionDecision,
    ) -> Result<SupervisionDecision, EventStoreError> {
        ensure_pending_resolution(self.supervision_decision(&decision.id)?, &decision)?;
        let affected = self.connection.execute(
            "UPDATE supervision_decisions
             SET decision_json = ?2
             WHERE decision_id = ?1",
            params![decision.id.0, serde_json::to_string(&decision)?],
        )?;
        if affected == 1 {
            Ok(decision)
        } else {
            Err(EventStoreError::SupervisionDecisionNotFound {
                decision_id: decision.id,
            })
        }
    }

    pub fn supervision_decisions_for_board(
        &self,
        board_id: &BoardId,
    ) -> Result<Vec<SupervisionDecision>, EventStoreError> {
        let mut statement = self.connection.prepare(
            "SELECT decision_json
             FROM supervision_decisions
             WHERE board_id = ?1
             ORDER BY recorded_at DESC, decision_id DESC
             LIMIT 50",
        )?;
        let rows = statement.query_map([board_id.0.as_str()], |row| row.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }

    fn supervision_decision(
        &self,
        decision_id: &SupervisionDecisionId,
    ) -> Result<Option<SupervisionDecision>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT decision_json FROM supervision_decisions WHERE decision_id = ?1",
                [decision_id.0.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|stored| Ok(serde_json::from_str(&stored)?))
            .transpose()
    }

    fn supervision_decision_by_key(
        &self,
        board_id: &BoardId,
        idempotency_key: &str,
    ) -> Result<Option<SupervisionDecision>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT decision_json
                 FROM supervision_decisions
                 WHERE board_id = ?1 AND idempotency_key = ?2",
                params![board_id.0, idempotency_key],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|stored| Ok(serde_json::from_str(&stored)?))
            .transpose()
    }
}

fn ensure_pending_resolution(
    recorded: Option<SupervisionDecision>,
    requested: &SupervisionDecision,
) -> Result<(), EventStoreError> {
    let Some(recorded) = recorded else {
        return Err(EventStoreError::SupervisionDecisionNotFound {
            decision_id: requested.id.clone(),
        });
    };
    let mut immutable_request = requested.clone();
    immutable_request.policy_result = recorded.policy_result;
    immutable_request.outcome = recorded.outcome;
    immutable_request.resolved_at = recorded.resolved_at.clone();
    if recorded.outcome == crate::domain::SupervisionDecisionOutcome::Pending
        && immutable_request == recorded
    {
        Ok(())
    } else {
        Err(EventStoreError::SupervisionDecisionConflict {
            decision_id: requested.id.clone(),
        })
    }
}

fn matching_decision(
    recorded: SupervisionDecision,
    requested: SupervisionDecision,
) -> Result<SupervisionDecision, EventStoreError> {
    if recorded == requested {
        Ok(recorded)
    } else {
        Err(EventStoreError::SupervisionDecisionConflict {
            decision_id: requested.id,
        })
    }
}
