mod contract;
mod fake_adapter;
mod ingestion;
mod process_adapter;
mod process_event_reader;
mod profile;

pub use contract::{
    AgentAdapter, AgentAdapterError, AgentCapabilities, AgentSession, NormalizedAgentEvent,
    NormalizedAgentEventKind, StartAgentRequest,
};
pub use fake_adapter::FakeAgentAdapter;
pub use ingestion::AgentEventIngestor;
pub use process_adapter::{ProcessAgentAdapter, ProcessAgentDefinition};
pub use profile::{AgentProfile, AgentProfileError};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod process_adapter_tests;
