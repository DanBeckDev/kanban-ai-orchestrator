use tempfile::TempDir;

use crate::{
    domain::PolicyDecisionKind,
    persistence::{EventStoreError, SqliteEventStore},
};

use super::sqlite_event_store_tests::policy_decision;

#[test]
fn rejects_databases_created_by_a_newer_schema_version() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let database_path = temporary_directory.path().join("newer-schema.sqlite");
    let connection =
        rusqlite::Connection::open(&database_path).expect("future database should open");
    connection
        .execute_batch(
            "CREATE TABLE schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        INSERT INTO schema_migrations (version, applied_at)
        VALUES (13, '2026-08-08T00:00:00Z');",
        )
        .expect("future schema version should be recorded");
    drop(connection);

    assert!(matches!(
        SqliteEventStore::open(&database_path),
        Err(EventStoreError::UnsupportedDatabaseSchemaVersion {
            current: 13,
            supported: 12
        })
    ));
}

#[test]
fn migrates_existing_event_stores_to_the_current_schema() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let database_path = temporary_directory.path().join("version-one.sqlite");
    let connection =
        rusqlite::Connection::open(&database_path).expect("version-one database should open");
    connection
        .execute_batch(
            "CREATE TABLE schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        INSERT INTO schema_migrations (version, applied_at) VALUES (1, '2026-08-08T00:00:00Z');
        CREATE TABLE work_item_events (
            sequence INTEGER PRIMARY KEY AUTOINCREMENT,
            event_id TEXT NOT NULL UNIQUE,
            work_item_id TEXT NOT NULL,
            event_json TEXT NOT NULL
        );
        CREATE TABLE materialized_work_items (
            work_item_id TEXT PRIMARY KEY,
            work_item_json TEXT NOT NULL,
            last_event_sequence INTEGER NOT NULL
        );",
        )
        .expect("version-one tables should be created");
    drop(connection);

    let mut store = SqliteEventStore::open(&database_path).expect("store should migrate");
    let decision = policy_decision(
        "migrated-policy-decision",
        PolicyDecisionKind::Deny,
        "daemon",
    );
    assert_eq!(
        store
            .database_schema_version()
            .expect("schema version should load"),
        12
    );
    assert_eq!(
        store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name IN ('executions', 'evidence', 'agent_profiles', 'planner_profiles', 'external_links', 'plan_proposals', 'connector_outbox_items', 'connector_reconciliation_items', 'board_access', 'project_agent_settings')",
                [],
                |row| row.get::<_, i64>(0),
            )
            .expect("execution tables should be created during migration"),
        10
    );
    assert_eq!(
        store
            .record_policy_decision(decision.clone())
            .expect("migrated table should accept policy decisions"),
        decision
    );
}
