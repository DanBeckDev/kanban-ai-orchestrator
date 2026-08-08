use rusqlite::{OptionalExtension, Transaction, params};

use crate::{
    application::StoredPlan,
    domain::{
        BoardId, PlanId, RecordedWorkItemEvent, SchemaMetadata, WorkItem, WorkItemEvent,
        WorkItemEventId, WorkItemEventKind,
    },
    orchestration::{PlanConfirmation, PlanProposal},
};

use super::{BoardStoreError, SqliteEventStore};

impl SqliteEventStore {
    pub fn save_plan_proposal(&mut self, proposal: PlanProposal) -> Result<(), BoardStoreError> {
        let board_id = proposal_board_id(&proposal)?;
        if let Some(recorded_plan) = self.stored_plan_for_board(&board_id)? {
            if recorded_plan.proposal == proposal {
                return Ok(());
            }
            if recorded_plan.confirmation.is_some() {
                return Err(BoardStoreError::PlanAlreadyExists { board_id });
            }
            if recorded_plan.proposal.id != proposal.id && self.plan_exists(&proposal.id)? {
                return Err(BoardStoreError::PlanIdConflict {
                    plan_id: proposal.id,
                });
            }
            self.connection.execute(
                "UPDATE plan_proposals
                 SET plan_id = ?1, proposal_json = ?2
                 WHERE board_id = ?3 AND confirmation_json IS NULL",
                params![proposal.id.0, serde_json::to_string(&proposal)?, board_id.0,],
            )?;
            return Ok(());
        }
        if self.plan_exists(&proposal.id)? {
            return Err(BoardStoreError::PlanIdConflict {
                plan_id: proposal.id,
            });
        }

        self.connection.execute(
            "INSERT INTO plan_proposals (plan_id, board_id, proposal_json, confirmation_json)
             VALUES (?1, ?2, ?3, NULL)",
            params![proposal.id.0, board_id.0, serde_json::to_string(&proposal)?,],
        )?;
        Ok(())
    }

    pub fn stored_plan_for_board(
        &self,
        board_id: &BoardId,
    ) -> Result<Option<StoredPlan>, BoardStoreError> {
        let stored = self
            .connection
            .query_row(
                "SELECT proposal_json, confirmation_json
                 FROM plan_proposals
                 WHERE board_id = ?1",
                [board_id.0.as_str()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .optional()?;
        stored
            .map(|(proposal_json, confirmation_json)| {
                Ok(StoredPlan {
                    proposal: serde_json::from_str(&proposal_json)?,
                    confirmation: confirmation_json
                        .map(|json| serde_json::from_str(&json))
                        .transpose()?,
                })
            })
            .transpose()
    }

    pub fn confirm_and_materialize_plan(
        &mut self,
        proposal: PlanProposal,
        confirmation: PlanConfirmation,
    ) -> Result<(), BoardStoreError> {
        let board_id = proposal_board_id(&proposal)?;
        let recorded_plan = self.stored_plan_for_board(&board_id)?.ok_or_else(|| {
            BoardStoreError::PlanNotFound {
                plan_id: proposal.id.clone(),
            }
        })?;
        if recorded_plan.proposal != proposal {
            return Err(BoardStoreError::PlanProposalConflict {
                plan_id: proposal.id,
            });
        }
        if let Some(recorded_confirmation) = recorded_plan.confirmation {
            return if recorded_confirmation == confirmation {
                Ok(())
            } else {
                Err(BoardStoreError::PlanConfirmationConflict {
                    plan_id: proposal.id,
                })
            };
        }

        let transaction = self.connection.transaction()?;
        ensure_plan_targets_are_available(&transaction, &proposal)?;
        for work_item in &proposal.work_items {
            persist_plan_work_item(&transaction, &proposal.id, work_item, &confirmation)?;
        }
        for dependency in &proposal.dependencies {
            transaction.execute(
                "INSERT INTO board_dependencies (
                    dependency_id,
                    board_id,
                    upstream_work_item_id,
                    downstream_work_item_id,
                    dependency_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    dependency.id.0,
                    board_id.0,
                    dependency.upstream_work_item_id.0,
                    dependency.downstream_work_item_id.0,
                    serde_json::to_string(dependency)?,
                ],
            )?;
        }
        let updated = transaction.execute(
            "UPDATE plan_proposals
             SET confirmation_json = ?1
             WHERE plan_id = ?2 AND confirmation_json IS NULL",
            params![serde_json::to_string(&confirmation)?, proposal.id.0],
        )?;
        if updated != 1 {
            return Err(BoardStoreError::PlanConfirmationConflict {
                plan_id: proposal.id,
            });
        }
        transaction.commit()?;
        Ok(())
    }

    fn plan_exists(&self, plan_id: &PlanId) -> Result<bool, BoardStoreError> {
        Ok(self
            .connection
            .query_row(
                "SELECT 1 FROM plan_proposals WHERE plan_id = ?1",
                [plan_id.0.as_str()],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }
}

pub(crate) fn create_plan_schema(transaction: &Transaction<'_>) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS plan_proposals (
            plan_id TEXT PRIMARY KEY,
            board_id TEXT NOT NULL UNIQUE,
            proposal_json TEXT NOT NULL,
            confirmation_json TEXT
        );",
    )
}

fn proposal_board_id(proposal: &PlanProposal) -> Result<BoardId, BoardStoreError> {
    proposal
        .work_items
        .first()
        .map(|work_item| work_item.board_id.clone())
        .ok_or_else(|| BoardStoreError::PlanNotFound {
            plan_id: proposal.id.clone(),
        })
}

fn ensure_plan_targets_are_available(
    transaction: &Transaction<'_>,
    proposal: &PlanProposal,
) -> Result<(), BoardStoreError> {
    for work_item in &proposal.work_items {
        let exists = transaction
            .query_row(
                "SELECT 1 FROM materialized_work_items WHERE work_item_id = ?1",
                [work_item.id.0.as_str()],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if exists {
            return Err(BoardStoreError::PlanWorkItemAlreadyExists {
                work_item_id: work_item.id.clone(),
            });
        }
    }
    for dependency in &proposal.dependencies {
        let exists = transaction
            .query_row(
                "SELECT 1 FROM board_dependencies WHERE dependency_id = ?1",
                [dependency.id.0.as_str()],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if exists {
            return Err(BoardStoreError::PlanDependencyAlreadyExists {
                dependency_id: dependency.id.clone(),
            });
        }
    }
    Ok(())
}

fn persist_plan_work_item(
    transaction: &Transaction<'_>,
    plan_id: &PlanId,
    work_item: &WorkItem,
    confirmation: &PlanConfirmation,
) -> Result<RecordedWorkItemEvent, BoardStoreError> {
    let event = WorkItemEvent {
        schema: SchemaMetadata::current(),
        id: WorkItemEventId::from(format!("plan-{}-create-{}", plan_id.0, work_item.id.0).as_str()),
        work_item_id: work_item.id.clone(),
        kind: WorkItemEventKind::Created {
            work_item: work_item.clone(),
        },
        recorded_at: confirmation.confirmed_at.clone(),
    };
    transaction.execute(
        "INSERT INTO work_item_events (event_id, work_item_id, event_json)
         VALUES (?1, ?2, ?3)",
        params![
            event.id.0,
            event.work_item_id.0,
            serde_json::to_string(&event)?,
        ],
    )?;
    let sequence = transaction.last_insert_rowid().try_into().map_err(|_| {
        BoardStoreError::from(super::EventStoreError::InvalidEventSequence {
            value: transaction.last_insert_rowid(),
        })
    })?;
    transaction.execute(
        "INSERT INTO materialized_work_items (
            work_item_id,
            work_item_json,
            last_event_sequence
         ) VALUES (?1, ?2, ?3)",
        params![
            work_item.id.0,
            serde_json::to_string(work_item)?,
            i64::try_from(sequence).expect("database event sequence is an i64"),
        ],
    )?;
    Ok(RecordedWorkItemEvent { sequence, event })
}
