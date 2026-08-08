use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::{
    CreateWorkItemCommand, MaterializedWorkItem, PolicyDecision, PolicyDecisionId, ProjectId,
    RecordedWorkItemEvent, SchemaMetadata, TransitionWorkItemCommand, WorkItem, WorkItemEvent,
    WorkItemEventId, WorkItemEventKind, WorkItemId, transition_work_item,
};
use crate::policy::ProtectedGitApproval;

use super::{
    EventStoreError,
    event_store_policy::{
        idempotent_policy_decision, idempotent_protected_git_approval,
        policy_decision_matches_protected_git_approval,
    },
    event_store_schema::{
        create_agent_profile_schema, create_execution_schema, create_external_link_schema,
        create_initial_schema, create_planner_profile_schema, create_policy_audit_schema,
    },
    event_store_support::{
        deserialize_recorded_event, event_sequence, idempotent_creation, idempotent_transition,
    },
};

const CURRENT_DATABASE_SCHEMA_VERSION: i64 = 9;
pub struct SqliteEventStore {
    pub(crate) connection: Connection,
}

impl SqliteEventStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, EventStoreError> {
        Self::from_connection(Connection::open(path)?)
    }

    pub fn in_memory() -> Result<Self, EventStoreError> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    pub fn database_schema_version(&self) -> Result<i64, EventStoreError> {
        Ok(self.connection.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?)
    }

    pub fn create_work_item(
        &mut self,
        command: CreateWorkItemCommand,
    ) -> Result<RecordedWorkItemEvent, EventStoreError> {
        if let Some(recorded_event) = self.event_by_id(&command.event_id)? {
            return idempotent_creation(recorded_event, &command);
        }

        if self
            .materialized_work_item(&command.work_item.id)?
            .is_some()
        {
            return Err(EventStoreError::WorkItemAlreadyExists {
                work_item_id: command.work_item.id,
            });
        }

        let event = WorkItemEvent {
            schema: SchemaMetadata::current(),
            id: command.event_id,
            work_item_id: command.work_item.id.clone(),
            kind: WorkItemEventKind::Created {
                work_item: command.work_item.clone(),
            },
            recorded_at: command.recorded_at,
        };

        self.persist_event(event, command.work_item)
    }

    pub fn transition_work_item(
        &mut self,
        command: TransitionWorkItemCommand,
    ) -> Result<RecordedWorkItemEvent, EventStoreError> {
        if let Some(recorded_event) = self.event_by_id(&command.event_id)? {
            return idempotent_transition(recorded_event, &command);
        }

        if command.reason.trim().is_empty() {
            return Err(EventStoreError::MissingTransitionReason {
                event_id: command.event_id,
            });
        }

        let materialized_work_item = self.required_materialized_work_item(&command.work_item_id)?;
        let next_state = transition_work_item(
            materialized_work_item.work_item.state,
            command.next_state,
            command.config,
            command.evidence,
        )?;
        let mut updated_work_item = materialized_work_item.work_item;
        let previous_state = updated_work_item.state;
        updated_work_item.state = next_state;
        let event = WorkItemEvent {
            schema: SchemaMetadata::current(),
            id: command.event_id,
            work_item_id: command.work_item_id,
            kind: WorkItemEventKind::StateTransitioned {
                from: previous_state,
                to: next_state,
                config: command.config,
                evidence: command.evidence,
                reason: command.reason,
            },
            recorded_at: command.recorded_at,
        };

        self.persist_event(event, updated_work_item)
    }

    pub fn record_policy_decision(
        &mut self,
        decision: PolicyDecision,
    ) -> Result<PolicyDecision, EventStoreError> {
        if let Some(recorded_decision) = self.policy_decision_by_id(&decision.id)? {
            return idempotent_policy_decision(recorded_decision, &decision);
        }

        let decision_json = serde_json::to_string(&decision)?;
        self.connection.execute(
            "INSERT INTO policy_decisions (
                decision_id,
                project_id,
                work_item_id,
                decided_at,
                decision_json
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                decision.id.0,
                decision.project_id.0,
                decision
                    .work_item_id
                    .as_ref()
                    .map(|work_item_id| work_item_id.0.as_str()),
                decision.decided_at,
                decision_json,
            ],
        )?;

        Ok(decision)
    }

    pub fn policy_decisions_for_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<PolicyDecision>, EventStoreError> {
        let mut statement = self.connection.prepare(
            "SELECT decision_json
             FROM policy_decisions
             WHERE project_id = ?1
             ORDER BY decided_at, decision_id",
        )?;
        let rows = statement.query_map([project_id.0.as_str()], |row| row.get::<_, String>(0))?;

        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }

    pub fn record_protected_git_approval(
        &mut self,
        approval: ProtectedGitApproval,
    ) -> Result<ProtectedGitApproval, EventStoreError> {
        if let Some(recorded_approval) = self.protected_git_approval_by_id(&approval.decision_id)? {
            return idempotent_protected_git_approval(recorded_approval, &approval);
        }

        let approval_decision = self
            .policy_decision_by_id(&approval.decision_id)?
            .ok_or_else(|| EventStoreError::PolicyApprovalDecisionNotFound {
                decision_id: approval.decision_id.clone(),
            })?;
        if !policy_decision_matches_protected_git_approval(&approval_decision, &approval) {
            return Err(EventStoreError::PolicyApprovalDecisionMismatch {
                decision_id: approval.decision_id,
            });
        }

        let approval_json = serde_json::to_string(&approval)?;
        self.connection.execute(
            "INSERT INTO protected_git_approvals (
                approval_decision_id,
                project_id,
                work_item_id,
                git_action,
                approved_at,
                approval_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                approval.decision_id.0,
                approval.project_id.0,
                approval
                    .work_item_id
                    .as_ref()
                    .map(|work_item_id| work_item_id.0.as_str()),
                approval.action.to_string(),
                approval.approved_at,
                approval_json,
            ],
        )?;

        Ok(approval)
    }

    pub fn has_recorded_protected_git_approval(
        &self,
        approval: &ProtectedGitApproval,
    ) -> Result<bool, EventStoreError> {
        Ok(self
            .protected_git_approval_by_id(&approval.decision_id)?
            .is_some_and(|recorded_approval| recorded_approval == *approval))
    }

    fn from_connection(connection: Connection) -> Result<Self, EventStoreError> {
        let mut store = Self { connection };
        store.apply_migrations()?;
        Ok(store)
    }

    fn apply_migrations(&mut self) -> Result<(), EventStoreError> {
        let transaction = self.connection.transaction()?;
        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );",
        )?;
        let current_version: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;

        if current_version > CURRENT_DATABASE_SCHEMA_VERSION {
            return Err(EventStoreError::UnsupportedDatabaseSchemaVersion {
                current: current_version,
                supported: CURRENT_DATABASE_SCHEMA_VERSION,
            });
        }

        if current_version < 1 {
            create_initial_schema(&transaction)?;
            transaction.execute("INSERT INTO schema_migrations (version) VALUES (?1)", [1])?;
        }

        if current_version < 2 {
            create_policy_audit_schema(&transaction)?;
            transaction.execute("INSERT INTO schema_migrations (version) VALUES (?1)", [2])?;
        }

        if current_version < 3 {
            crate::persistence::board_store::create_protected_git_approval_schema(&transaction)?;
            transaction.execute("INSERT INTO schema_migrations (version) VALUES (?1)", [3])?;
        }

        if current_version < 4 {
            crate::persistence::board_store::create_board_schema(&transaction)?;
            transaction.execute("INSERT INTO schema_migrations (version) VALUES (?1)", [4])?;
        }

        if current_version < 5 {
            create_execution_schema(&transaction)?;
            transaction.execute("INSERT INTO schema_migrations (version) VALUES (?1)", [5])?;
        }

        if current_version < 6 {
            create_agent_profile_schema(&transaction)?;
            transaction.execute("INSERT INTO schema_migrations (version) VALUES (?1)", [6])?;
        }

        if current_version < 7 {
            create_external_link_schema(&transaction)?;
            transaction.execute("INSERT INTO schema_migrations (version) VALUES (?1)", [7])?;
        }

        if current_version < 8 {
            crate::persistence::plan_store::create_plan_schema(&transaction)?;
            transaction.execute("INSERT INTO schema_migrations (version) VALUES (?1)", [8])?;
        }

        if current_version < 9 {
            create_planner_profile_schema(&transaction)?;
            transaction.execute("INSERT INTO schema_migrations (version) VALUES (?1)", [9])?;
        }

        transaction.commit()?;
        Ok(())
    }

    fn event_by_id(
        &self,
        event_id: &WorkItemEventId,
    ) -> Result<Option<RecordedWorkItemEvent>, EventStoreError> {
        let stored_event = self
            .connection
            .query_row(
                "SELECT sequence, event_json
                 FROM work_item_events
                 WHERE event_id = ?1",
                [event_id.0.as_str()],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;

        stored_event
            .map(|(sequence, event_json)| deserialize_recorded_event(&event_json, sequence))
            .transpose()
    }

    fn policy_decision_by_id(
        &self,
        decision_id: &PolicyDecisionId,
    ) -> Result<Option<PolicyDecision>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT decision_json
                 FROM policy_decisions
                 WHERE decision_id = ?1",
                [decision_id.0.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|decision_json| Ok(serde_json::from_str(&decision_json)?))
            .transpose()
    }

    fn protected_git_approval_by_id(
        &self,
        decision_id: &PolicyDecisionId,
    ) -> Result<Option<ProtectedGitApproval>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT approval_json
                 FROM protected_git_approvals
                 WHERE approval_decision_id = ?1",
                [decision_id.0.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|approval_json| Ok(serde_json::from_str(&approval_json)?))
            .transpose()
    }

    pub(super) fn required_materialized_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<MaterializedWorkItem, EventStoreError> {
        self.materialized_work_item(work_item_id)?.ok_or_else(|| {
            EventStoreError::WorkItemNotFound {
                work_item_id: work_item_id.clone(),
            }
        })
    }

    fn persist_event(
        &mut self,
        event: WorkItemEvent,
        work_item: WorkItem,
    ) -> Result<RecordedWorkItemEvent, EventStoreError> {
        let event_json = serde_json::to_string(&event)?;
        let work_item_json = serde_json::to_string(&work_item)?;
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT INTO work_item_events (event_id, work_item_id, event_json)
             VALUES (?1, ?2, ?3)",
            params![event.id.0, event.work_item_id.0, event_json],
        )?;
        let database_sequence = transaction.last_insert_rowid();
        let sequence = event_sequence(database_sequence)?;
        transaction.execute(
            "INSERT INTO materialized_work_items (
                work_item_id,
                work_item_json,
                last_event_sequence
             ) VALUES (?1, ?2, ?3)
             ON CONFLICT(work_item_id) DO UPDATE SET
                work_item_json = excluded.work_item_json,
                last_event_sequence = excluded.last_event_sequence",
            params![work_item.id.0, work_item_json, database_sequence],
        )?;
        transaction.commit()?;

        Ok(RecordedWorkItemEvent { sequence, event })
    }
}

#[cfg(test)]
mod tests {
    use crate::persistence::event_store_support::event_sequence;

    #[test]
    fn rejects_negative_database_sequences() {
        assert!(event_sequence(-1).is_err());
    }
}
