use rusqlite::Transaction;

pub(super) fn create_initial_schema(transaction: &Transaction<'_>) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS work_item_events (
            sequence INTEGER PRIMARY KEY AUTOINCREMENT,
            event_id TEXT NOT NULL UNIQUE,
            work_item_id TEXT NOT NULL,
            event_json TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS work_item_events_by_work_item ON work_item_events (work_item_id, sequence);
        CREATE TABLE IF NOT EXISTS materialized_work_items (
            work_item_id TEXT PRIMARY KEY,
            work_item_json TEXT NOT NULL,
            last_event_sequence INTEGER NOT NULL
        );",
    )
}

pub(super) fn create_policy_audit_schema(
    transaction: &Transaction<'_>,
) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS policy_decisions (
            decision_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            work_item_id TEXT,
            decided_at TEXT NOT NULL,
            decision_json TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS policy_decisions_by_project ON policy_decisions (project_id, decided_at, decision_id);",
    )
}

pub(super) fn create_execution_schema(
    transaction: &Transaction<'_>,
) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS executions (
            execution_id TEXT PRIMARY KEY,
            work_item_id TEXT NOT NULL,
            execution_json TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS executions_by_work_item ON executions (work_item_id, execution_id);
        CREATE TABLE IF NOT EXISTS evidence (
            evidence_id TEXT PRIMARY KEY,
            work_item_id TEXT NOT NULL,
            evidence_json TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS evidence_by_work_item ON evidence (work_item_id, evidence_id);",
    )
}

pub(super) fn create_agent_profile_schema(
    transaction: &Transaction<'_>,
) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS agent_profiles (
            profile_name TEXT PRIMARY KEY,
            profile_json TEXT NOT NULL
        );",
    )
}

pub(super) fn create_external_link_schema(
    transaction: &Transaction<'_>,
) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS external_links (
            external_link_id TEXT PRIMARY KEY,
            work_item_id TEXT NOT NULL,
            connector_id TEXT NOT NULL,
            external_id TEXT NOT NULL,
            link_json TEXT NOT NULL,
            UNIQUE(connector_id, external_id)
        );
        CREATE INDEX IF NOT EXISTS external_links_by_work_item
            ON external_links (work_item_id, external_link_id);",
    )
}
