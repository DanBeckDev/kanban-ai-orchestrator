use rusqlite::{OptionalExtension, params, params_from_iter};

use crate::domain::{Evidence, EvidenceId, Execution, ExecutionId, ExecutionStatus, WorkItemId};

use super::{EventStoreError, SqliteEventStore};

impl SqliteEventStore {
    pub fn record_execution(&mut self, execution: Execution) -> Result<Execution, EventStoreError> {
        if self
            .materialized_work_item(&execution.work_item_id)?
            .is_none()
        {
            return Err(EventStoreError::WorkItemNotFound {
                work_item_id: execution.work_item_id,
            });
        }
        if let Some(recorded_execution) = self.execution(&execution.id)? {
            return if recorded_execution == execution {
                Ok(recorded_execution)
            } else {
                Err(EventStoreError::ExecutionAlreadyExists {
                    execution_id: execution.id,
                })
            };
        }

        self.connection.execute(
            "INSERT INTO executions (execution_id, work_item_id, execution_json)
             VALUES (?1, ?2, ?3)",
            params![
                execution.id.0,
                execution.work_item_id.0,
                serde_json::to_string(&execution)?,
            ],
        )?;
        Ok(execution)
    }

    pub fn execution(
        &self,
        execution_id: &ExecutionId,
    ) -> Result<Option<Execution>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT execution_json FROM executions WHERE execution_id = ?1",
                [execution_id.0.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|execution_json| Ok(serde_json::from_str(&execution_json)?))
            .transpose()
    }

    pub fn update_execution(&mut self, execution: Execution) -> Result<Execution, EventStoreError> {
        let recorded_execution =
            self.execution(&execution.id)?
                .ok_or_else(|| EventStoreError::ExecutionNotFound {
                    execution_id: execution.id.clone(),
                })?;
        if recorded_execution == execution {
            return Ok(recorded_execution);
        }
        validate_execution_update(&recorded_execution, &execution)?;

        self.connection.execute(
            "UPDATE executions SET execution_json = ?1 WHERE execution_id = ?2",
            params![serde_json::to_string(&execution)?, execution.id.0],
        )?;
        Ok(execution)
    }

    pub fn executions_for_work_items(
        &self,
        work_item_ids: &[WorkItemId],
    ) -> Result<Vec<Execution>, EventStoreError> {
        query_records_for_work_items(
            &self.connection,
            "executions",
            "execution_json",
            work_item_ids,
        )
    }

    pub fn recent_executions_for_work_items(
        &self,
        work_item_ids: &[WorkItemId],
        limit_per_work_item: u32,
    ) -> Result<Vec<Execution>, EventStoreError> {
        recent_records_for_work_items(
            &self.connection,
            "executions",
            "execution_json",
            work_item_ids,
            limit_per_work_item,
        )
    }

    pub fn record_evidence(&mut self, evidence: Evidence) -> Result<Evidence, EventStoreError> {
        if self
            .materialized_work_item(&evidence.work_item_id)?
            .is_none()
        {
            return Err(EventStoreError::WorkItemNotFound {
                work_item_id: evidence.work_item_id,
            });
        }
        if let Some(recorded_evidence) = self.evidence(&evidence.id)? {
            return if recorded_evidence == evidence {
                Ok(recorded_evidence)
            } else {
                Err(EventStoreError::EvidenceAlreadyExists {
                    evidence_id: evidence.id,
                })
            };
        }

        self.connection.execute(
            "INSERT INTO evidence (evidence_id, work_item_id, evidence_json)
             VALUES (?1, ?2, ?3)",
            params![
                evidence.id.0,
                evidence.work_item_id.0,
                serde_json::to_string(&evidence)?,
            ],
        )?;
        Ok(evidence)
    }

    pub fn evidence(&self, evidence_id: &EvidenceId) -> Result<Option<Evidence>, EventStoreError> {
        self.connection
            .query_row(
                "SELECT evidence_json FROM evidence WHERE evidence_id = ?1",
                [evidence_id.0.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|evidence_json| Ok(serde_json::from_str(&evidence_json)?))
            .transpose()
    }

    pub fn evidence_for_work_items(
        &self,
        work_item_ids: &[WorkItemId],
    ) -> Result<Vec<Evidence>, EventStoreError> {
        query_records_for_work_items(&self.connection, "evidence", "evidence_json", work_item_ids)
    }

    pub fn recent_evidence_for_work_items(
        &self,
        work_item_ids: &[WorkItemId],
        limit_per_work_item: u32,
    ) -> Result<Vec<Evidence>, EventStoreError> {
        recent_records_for_work_items(
            &self.connection,
            "evidence",
            "evidence_json",
            work_item_ids,
            limit_per_work_item,
        )
    }
}

fn query_records_for_work_items<Record>(
    connection: &rusqlite::Connection,
    table: &str,
    json_column: &str,
    work_item_ids: &[WorkItemId],
) -> Result<Vec<Record>, EventStoreError>
where
    Record: serde::de::DeserializeOwned,
{
    if work_item_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = std::iter::repeat_n("?", work_item_ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT {json_column} FROM {table}
         WHERE work_item_id IN ({placeholders})
         ORDER BY work_item_id, rowid"
    );
    let mut statement = connection.prepare(&sql)?;
    let parameters = work_item_ids
        .iter()
        .map(|work_item_id| work_item_id.0.as_str());
    let rows = statement.query_map(params_from_iter(parameters), |row| row.get::<_, String>(0))?;

    rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
}

fn recent_records_for_work_items<Record>(
    connection: &rusqlite::Connection,
    table: &str,
    json_column: &str,
    work_item_ids: &[WorkItemId],
    limit_per_work_item: u32,
) -> Result<Vec<Record>, EventStoreError>
where
    Record: serde::de::DeserializeOwned,
{
    if limit_per_work_item == 0 {
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    let sql = format!(
        "SELECT {json_column} FROM {table}
         WHERE work_item_id = ?1
         ORDER BY rowid DESC
         LIMIT ?2"
    );
    let mut statement = connection.prepare(&sql)?;
    for work_item_id in work_item_ids {
        let rows = statement.query_map(
            params![work_item_id.0.as_str(), i64::from(limit_per_work_item)],
            |row| row.get::<_, String>(0),
        )?;
        let mut work_item_records = rows
            .map(|row| Ok(serde_json::from_str::<Record>(&row?)?))
            .collect::<Result<Vec<_>, EventStoreError>>()?;
        work_item_records.reverse();
        records.extend(work_item_records);
    }
    Ok(records)
}

fn validate_execution_update(
    recorded_execution: &Execution,
    updated_execution: &Execution,
) -> Result<(), EventStoreError> {
    let execution_id = updated_execution.id.clone();
    if recorded_execution.schema != updated_execution.schema
        || recorded_execution.work_item_id != updated_execution.work_item_id
        || recorded_execution.adapter_name != updated_execution.adapter_name
        || recorded_execution.workspace_path != updated_execution.workspace_path
    {
        return Err(EventStoreError::InvalidExecutionUpdate {
            execution_id,
            reason: "execution identity is immutable",
        });
    }
    if updated_execution
        .session_id
        .as_deref()
        .is_some_and(|session_id| session_id.trim().is_empty())
    {
        return Err(EventStoreError::InvalidExecutionUpdate {
            execution_id,
            reason: "session identity cannot be blank",
        });
    }
    if !session_is_consistent(recorded_execution, updated_execution) {
        return Err(EventStoreError::InvalidExecutionUpdate {
            execution_id,
            reason: "session identity cannot be replaced or removed",
        });
    }
    if recorded_execution.status == ExecutionStatus::Pending
        && updated_execution.status == ExecutionStatus::Running
        && updated_execution.session_id.is_none()
    {
        return Err(EventStoreError::InvalidExecutionUpdate {
            execution_id,
            reason: "a running execution requires an attached session",
        });
    }
    if !usage_is_monotonic(recorded_execution, updated_execution) {
        return Err(EventStoreError::InvalidExecutionUpdate {
            execution_id,
            reason: "usage cannot decrease or discard recorded cost",
        });
    }
    if updated_execution.last_event_sequence < recorded_execution.last_event_sequence {
        return Err(EventStoreError::InvalidExecutionUpdate {
            execution_id,
            reason: "event sequence cannot decrease",
        });
    }
    if recorded_execution.status != updated_execution.status
        && !recorded_execution
            .status
            .allows_transition_to(updated_execution.status)
    {
        return Err(EventStoreError::InvalidExecutionUpdate {
            execution_id,
            reason: "status transition is not allowed",
        });
    }
    Ok(())
}

fn session_is_consistent(recorded_execution: &Execution, updated_execution: &Execution) -> bool {
    match (
        &recorded_execution.session_id,
        &updated_execution.session_id,
    ) {
        (None, _) => true,
        (Some(recorded), Some(updated)) => recorded == updated,
        _ => false,
    }
}

fn usage_is_monotonic(recorded_execution: &Execution, updated_execution: &Execution) -> bool {
    let recorded_usage = &recorded_execution.usage;
    let updated_usage = &updated_execution.usage;
    let cost_is_monotonic = match (recorded_usage.cost_micros, updated_usage.cost_micros) {
        (None, _) => true,
        (Some(recorded), Some(updated)) => recorded <= updated,
        (Some(_), None) => false,
    };
    recorded_usage.input_tokens <= updated_usage.input_tokens
        && recorded_usage.output_tokens <= updated_usage.output_tokens
        && cost_is_monotonic
}
