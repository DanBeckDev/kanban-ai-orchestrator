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
