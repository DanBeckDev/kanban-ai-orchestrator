pub(crate) mod sqlite_event_store;

pub use sqlite_event_store::{EventStoreError, SqliteEventStore};

#[cfg(test)]
mod sqlite_event_store_tests;
