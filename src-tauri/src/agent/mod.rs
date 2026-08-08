mod contract;
mod fake_adapter;
mod ingestion;

pub use contract::{
    AgentAdapter, AgentAdapterError, AgentCapabilities, AgentSession, NormalizedAgentEvent,
    NormalizedAgentEventKind, StartAgentRequest,
};
pub use fake_adapter::FakeAgentAdapter;
pub use ingestion::AgentEventIngestor;

#[cfg(test)]
mod tests;
