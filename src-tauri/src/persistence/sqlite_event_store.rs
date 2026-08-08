use std::{error::Error, fmt, path::Path};

use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::domain::{
    CreateWorkItemCommand, EventSequence, MaterializedWorkItem, PolicyDecision, PolicyDecisionId,
    PolicyDecisionKind, ProjectId, RecordedWorkItemEvent, RestartReconciliationCommand,
    SchemaMetadata, TransitionWorkItemCommand, WorkItem, WorkItemEvent, WorkItemEventId,
    WorkItemEventKind, WorkItemId, WorkItemState, transition_work_item,
};
use crate::policy::ProtectedGitApproval;

const CURRENT_DATABASE_SCHEMA_VERSION: i64 = 3;
const RESTART_UNCERTAINTY_REASON: &str =
    "The daemon restarted before a live execution could be confirmed.";

pub struct SqliteEventStore {
    connection: Connection,
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

    pub fn reconcile_after_restart(
        &mut self,
        command: RestartReconciliationCommand,
    ) -> Result<Vec<RecordedWorkItemEvent>, EventStoreError> {
        let uncertain_work_items = self
            .all_materialized_work_items()?
            .into_iter()
            .filter(|materialized_work_item| {
                is_uncertain_after_restart(materialized_work_item.work_item.state)
                    && !command
                        .confirmed_active_work_item_ids
                        .contains(&materialized_work_item.work_item.id)
            })
            .collect::<Vec<_>>();

        for materialized_work_item in &uncertain_work_items {
            if !command
                .recovery_event_ids
                .contains_key(&materialized_work_item.work_item.id)
            {
                return Err(EventStoreError::MissingRecoveryEventId {
                    work_item_id: materialized_work_item.work_item.id.clone(),
                });
            }
        }

        uncertain_work_items
            .into_iter()
            .map(|materialized_work_item| {
                let event_id = command
                    .recovery_event_ids
                    .get(&materialized_work_item.work_item.id)
                    .cloned()
                    .ok_or_else(|| EventStoreError::MissingRecoveryEventId {
                        work_item_id: materialized_work_item.work_item.id.clone(),
                    })?;
                self.transition_work_item(TransitionWorkItemCommand {
                    event_id,
                    work_item_id: materialized_work_item.work_item.id,
                    next_state: WorkItemState::Interrupted,
                    config: Default::default(),
                    evidence: None,
                    reason: RESTART_UNCERTAINTY_REASON.to_owned(),
                    recorded_at: command.recorded_at.clone(),
                })
            })
            .collect()
    }

    pub fn materialized_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Option<MaterializedWorkItem>, EventStoreError> {
        let stored_work_item = self
            .connection
            .query_row(
                "SELECT work_item_json, last_event_sequence
                 FROM materialized_work_items
                 WHERE work_item_id = ?1",
                [work_item_id.0.as_str()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()?;

        stored_work_item
            .map(|(work_item_json, last_event_sequence)| {
                deserialize_materialized_work_item(&work_item_json, last_event_sequence)
            })
            .transpose()
    }

    pub fn work_item_events(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Vec<RecordedWorkItemEvent>, EventStoreError> {
        let mut statement = self.connection.prepare(
            "SELECT sequence, event_json
             FROM work_item_events
             WHERE work_item_id = ?1
             ORDER BY sequence",
        )?;
        let rows = statement.query_map([work_item_id.0.as_str()], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        rows.map(|row| {
            let (sequence, event_json) = row?;
            deserialize_recorded_event(&event_json, sequence)
        })
        .collect()
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
            create_protected_git_approval_schema(&transaction)?;
            transaction.execute("INSERT INTO schema_migrations (version) VALUES (?1)", [3])?;
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

    fn required_materialized_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<MaterializedWorkItem, EventStoreError> {
        self.materialized_work_item(work_item_id)?.ok_or_else(|| {
            EventStoreError::WorkItemNotFound {
                work_item_id: work_item_id.clone(),
            }
        })
    }

    fn all_materialized_work_items(&self) -> Result<Vec<MaterializedWorkItem>, EventStoreError> {
        let mut statement = self.connection.prepare(
            "SELECT work_item_json, last_event_sequence
             FROM materialized_work_items
             ORDER BY work_item_id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        rows.map(|row| {
            let (work_item_json, last_event_sequence) = row?;
            deserialize_materialized_work_item(&work_item_json, last_event_sequence)
        })
        .collect()
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

fn create_initial_schema(transaction: &Transaction<'_>) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS work_item_events (
            sequence INTEGER PRIMARY KEY AUTOINCREMENT,
            event_id TEXT NOT NULL UNIQUE,
            work_item_id TEXT NOT NULL,
            event_json TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS work_item_events_by_work_item
            ON work_item_events (work_item_id, sequence);
        CREATE TABLE IF NOT EXISTS materialized_work_items (
            work_item_id TEXT PRIMARY KEY,
            work_item_json TEXT NOT NULL,
            last_event_sequence INTEGER NOT NULL
        );",
    )
}

fn create_policy_audit_schema(transaction: &Transaction<'_>) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS policy_decisions (
            decision_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            work_item_id TEXT,
            decided_at TEXT NOT NULL,
            decision_json TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS policy_decisions_by_project
            ON policy_decisions (project_id, decided_at, decision_id);",
    )
}

fn create_protected_git_approval_schema(
    transaction: &Transaction<'_>,
) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS protected_git_approvals (
            approval_decision_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            work_item_id TEXT,
            git_action TEXT NOT NULL,
            approved_at TEXT NOT NULL,
            approval_json TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS protected_git_approvals_by_project
            ON protected_git_approvals (project_id, approved_at, approval_decision_id);",
    )
}

fn idempotent_creation(
    recorded_event: RecordedWorkItemEvent,
    command: &CreateWorkItemCommand,
) -> Result<RecordedWorkItemEvent, EventStoreError> {
    match &recorded_event.event.kind {
        WorkItemEventKind::Created { work_item }
            if recorded_event.event.work_item_id == command.work_item.id
                && work_item == &command.work_item =>
        {
            Ok(recorded_event)
        }
        _ => Err(EventStoreError::EventIdConflict {
            event_id: command.event_id.clone(),
        }),
    }
}

fn idempotent_transition(
    recorded_event: RecordedWorkItemEvent,
    command: &TransitionWorkItemCommand,
) -> Result<RecordedWorkItemEvent, EventStoreError> {
    match &recorded_event.event.kind {
        WorkItemEventKind::StateTransitioned {
            to,
            config,
            evidence,
            reason,
            ..
        } if recorded_event.event.work_item_id == command.work_item_id
            && *to == command.next_state
            && *config == command.config
            && *evidence == command.evidence
            && reason == &command.reason =>
        {
            Ok(recorded_event)
        }
        _ => Err(EventStoreError::EventIdConflict {
            event_id: command.event_id.clone(),
        }),
    }
}

fn idempotent_policy_decision(
    recorded_decision: PolicyDecision,
    decision: &PolicyDecision,
) -> Result<PolicyDecision, EventStoreError> {
    if recorded_decision == *decision {
        Ok(recorded_decision)
    } else {
        Err(EventStoreError::PolicyDecisionIdConflict {
            decision_id: decision.id.clone(),
        })
    }
}

fn idempotent_protected_git_approval(
    recorded_approval: ProtectedGitApproval,
    approval: &ProtectedGitApproval,
) -> Result<ProtectedGitApproval, EventStoreError> {
    if recorded_approval == *approval {
        Ok(recorded_approval)
    } else {
        Err(EventStoreError::ProtectedGitApprovalIdConflict {
            decision_id: approval.decision_id.clone(),
        })
    }
}

fn policy_decision_matches_protected_git_approval(
    decision: &PolicyDecision,
    approval: &ProtectedGitApproval,
) -> bool {
    decision.id == approval.decision_id
        && decision.project_id == approval.project_id
        && decision.work_item_id == approval.work_item_id
        && decision.action
            == Some(crate::domain::PolicyAction::ProtectedGit {
                action: approval.action,
            })
        && decision.decision == PolicyDecisionKind::Allow
        && decision.actor == approval.approved_by
}

fn is_uncertain_after_restart(state: WorkItemState) -> bool {
    matches!(state, WorkItemState::Running | WorkItemState::AwaitingInput)
}

fn deserialize_materialized_work_item(
    work_item_json: &str,
    last_event_sequence: i64,
) -> Result<MaterializedWorkItem, EventStoreError> {
    Ok(MaterializedWorkItem {
        work_item: serde_json::from_str(work_item_json)?,
        last_event_sequence: event_sequence(last_event_sequence)?,
    })
}

fn deserialize_recorded_event(
    event_json: &str,
    sequence: i64,
) -> Result<RecordedWorkItemEvent, EventStoreError> {
    Ok(RecordedWorkItemEvent {
        sequence: event_sequence(sequence)?,
        event: serde_json::from_str(event_json)?,
    })
}

fn event_sequence(value: i64) -> Result<EventSequence, EventStoreError> {
    u64::try_from(value).map_err(|_| EventStoreError::InvalidEventSequence { value })
}

#[derive(Debug)]
pub enum EventStoreError {
    Database(rusqlite::Error),
    Serialization(serde_json::Error),
    StateTransition(crate::domain::TransitionError),
    WorkItemAlreadyExists { work_item_id: WorkItemId },
    WorkItemNotFound { work_item_id: WorkItemId },
    EventIdConflict { event_id: WorkItemEventId },
    PolicyDecisionIdConflict { decision_id: PolicyDecisionId },
    ProtectedGitApprovalIdConflict { decision_id: PolicyDecisionId },
    PolicyApprovalDecisionNotFound { decision_id: PolicyDecisionId },
    PolicyApprovalDecisionMismatch { decision_id: PolicyDecisionId },
    MissingTransitionReason { event_id: WorkItemEventId },
    MissingRecoveryEventId { work_item_id: WorkItemId },
    InvalidEventSequence { value: i64 },
    UnsupportedDatabaseSchemaVersion { current: i64, supported: i64 },
}

impl fmt::Display for EventStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "SQLite event-store error: {error}"),
            Self::Serialization(error) => {
                write!(formatter, "event-store serialization error: {error}")
            }
            Self::StateTransition(error) => write!(formatter, "state transition rejected: {error}"),
            Self::WorkItemAlreadyExists { work_item_id } => {
                write!(formatter, "work item {} already exists", work_item_id.0)
            }
            Self::WorkItemNotFound { work_item_id } => {
                write!(formatter, "work item {} was not found", work_item_id.0)
            }
            Self::EventIdConflict { event_id } => {
                write!(
                    formatter,
                    "event id {} conflicts with a recorded event",
                    event_id.0
                )
            }
            Self::PolicyDecisionIdConflict { decision_id } => write!(
                formatter,
                "policy decision id {} conflicts with a recorded decision",
                decision_id.0
            ),
            Self::ProtectedGitApprovalIdConflict { decision_id } => write!(
                formatter,
                "protected Git approval id {} conflicts with a recorded approval",
                decision_id.0
            ),
            Self::PolicyApprovalDecisionNotFound { decision_id } => write!(
                formatter,
                "protected Git approval requires recorded policy decision {}",
                decision_id.0
            ),
            Self::PolicyApprovalDecisionMismatch { decision_id } => write!(
                formatter,
                "policy decision {} does not authorize the protected Git approval",
                decision_id.0
            ),
            Self::MissingTransitionReason { event_id } => {
                write!(
                    formatter,
                    "state-transition event {} requires a reason",
                    event_id.0
                )
            }
            Self::MissingRecoveryEventId { work_item_id } => write!(
                formatter,
                "restart reconciliation requires an event id for uncertain work item {}",
                work_item_id.0
            ),
            Self::InvalidEventSequence { value } => {
                write!(
                    formatter,
                    "event sequence {value} is outside the supported range"
                )
            }
            Self::UnsupportedDatabaseSchemaVersion { current, supported } => write!(
                formatter,
                "database schema version {current} is newer than the supported version {supported}"
            ),
        }
    }
}

impl Error for EventStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Serialization(error) => Some(error),
            Self::StateTransition(error) => Some(error),
            Self::WorkItemAlreadyExists { .. }
            | Self::WorkItemNotFound { .. }
            | Self::EventIdConflict { .. }
            | Self::PolicyDecisionIdConflict { .. }
            | Self::ProtectedGitApprovalIdConflict { .. }
            | Self::PolicyApprovalDecisionNotFound { .. }
            | Self::PolicyApprovalDecisionMismatch { .. }
            | Self::MissingTransitionReason { .. }
            | Self::MissingRecoveryEventId { .. }
            | Self::InvalidEventSequence { .. }
            | Self::UnsupportedDatabaseSchemaVersion { .. } => None,
        }
    }
}

impl From<rusqlite::Error> for EventStoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

impl From<serde_json::Error> for EventStoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

impl From<crate::domain::TransitionError> for EventStoreError {
    fn from(error: crate::domain::TransitionError) -> Self {
        Self::StateTransition(error)
    }
}

impl crate::policy::PolicyAuditStore for SqliteEventStore {
    type Error = EventStoreError;

    fn record_policy_decision(&mut self, decision: PolicyDecision) -> Result<(), Self::Error> {
        SqliteEventStore::record_policy_decision(self, decision).map(|_| ())
    }

    fn has_recorded_protected_git_approval(
        &self,
        approval: &ProtectedGitApproval,
    ) -> Result<bool, Self::Error> {
        SqliteEventStore::has_recorded_protected_git_approval(self, approval)
    }
}

#[cfg(test)]
mod tests {
    use super::event_sequence;

    #[test]
    fn rejects_negative_database_sequences() {
        assert!(event_sequence(-1).is_err());
    }
}
