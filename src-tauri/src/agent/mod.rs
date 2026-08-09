mod contract;
mod fake_adapter;
mod ingestion;
mod process_adapter;
mod process_event_reader;
mod profile;
mod provider_adapter;
mod provider_event_decoder;

pub use contract::{
    AgentAdapter, AgentAdapterError, AgentCapabilities, AgentSession, NormalizedAgentEvent,
    NormalizedAgentEventKind, StartAgentRequest,
};
pub use fake_adapter::FakeAgentAdapter;
pub use ingestion::AgentEventIngestor;
pub use process_adapter::{ProcessAgentAdapter, ProcessAgentDefinition};
pub use profile::{AgentProfile, AgentProfileError, AgentProfileKind};
pub use provider_adapter::{NativeProcessAdapter, WorkerAgentAdapter};

#[cfg(test)]
mod tests;

#[cfg(all(test, unix))]
mod conformance_tests;

#[cfg(all(test, unix))]
mod process_adapter_tests;

#[cfg(all(test, unix))]
mod provider_adapter_tests;

#[cfg(test)]
mod provider_event_decoder_tests;
