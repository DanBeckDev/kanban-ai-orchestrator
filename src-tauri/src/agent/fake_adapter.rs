use std::collections::BTreeMap;

use super::{
    AgentAdapter, AgentAdapterError, AgentCapabilities, AgentSession, NormalizedAgentEvent,
    StartAgentRequest,
};

pub struct FakeAgentAdapter {
    name: String,
    capabilities: AgentCapabilities,
    next_session_number: u64,
    sessions: BTreeMap<String, FakeSession>,
}

#[derive(Default)]
struct FakeSession {
    events: Vec<NormalizedAgentEvent>,
    feedback: Vec<String>,
    terminated: bool,
}

impl FakeAgentAdapter {
    pub fn new(name: impl Into<String>, capabilities: AgentCapabilities) -> Self {
        Self {
            name: name.into(),
            capabilities,
            next_session_number: 1,
            sessions: BTreeMap::new(),
        }
    }

    pub fn queue_event(
        &mut self,
        session_id: &str,
        event: NormalizedAgentEvent,
    ) -> Result<(), AgentAdapterError> {
        self.session_mut(session_id)?.events.push(event);
        Ok(())
    }

    pub fn feedback(&self, session_id: &str) -> Result<&[String], AgentAdapterError> {
        Ok(&self.session(session_id)?.feedback)
    }

    fn session(&self, session_id: &str) -> Result<&FakeSession, AgentAdapterError> {
        self.sessions
            .get(session_id)
            .ok_or_else(|| AgentAdapterError::UnknownSession {
                session_id: session_id.to_owned(),
            })
    }

    fn session_mut(&mut self, session_id: &str) -> Result<&mut FakeSession, AgentAdapterError> {
        self.sessions
            .get_mut(session_id)
            .ok_or_else(|| AgentAdapterError::UnknownSession {
                session_id: session_id.to_owned(),
            })
    }
}

impl AgentAdapter for FakeAgentAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn discover(&self) -> Result<AgentCapabilities, AgentAdapterError> {
        Ok(self.capabilities)
    }

    fn start(&mut self, _request: StartAgentRequest) -> Result<AgentSession, AgentAdapterError> {
        let id = format!("fake-session-{}", self.next_session_number);
        self.next_session_number += 1;
        self.sessions.insert(id.clone(), FakeSession::default());
        Ok(AgentSession {
            id,
            resumable: self.capabilities.supports_resume,
        })
    }

    fn resume(&mut self, session_id: &str) -> Result<AgentSession, AgentAdapterError> {
        if !self.capabilities.supports_resume {
            return Err(AgentAdapterError::CapabilityUnsupported {
                capability: "resume",
            });
        }
        self.session(session_id)?;
        Ok(AgentSession {
            id: session_id.to_owned(),
            resumable: true,
        })
    }

    fn send_feedback(&mut self, session_id: &str, feedback: &str) -> Result<(), AgentAdapterError> {
        if !self.capabilities.supports_feedback {
            return Err(AgentAdapterError::CapabilityUnsupported {
                capability: "feedback",
            });
        }
        self.session_mut(session_id)?
            .feedback
            .push(feedback.to_owned());
        Ok(())
    }

    fn interrupt(&mut self, session_id: &str) -> Result<(), AgentAdapterError> {
        if !self.capabilities.supports_interrupt {
            return Err(AgentAdapterError::CapabilityUnsupported {
                capability: "interrupt",
            });
        }
        self.session_mut(session_id)?;
        Ok(())
    }

    fn terminate(&mut self, session_id: &str) -> Result<(), AgentAdapterError> {
        self.session_mut(session_id)?.terminated = true;
        Ok(())
    }

    fn stream_events(
        &self,
        session_id: &str,
        after_sequence: u64,
    ) -> Result<Vec<NormalizedAgentEvent>, AgentAdapterError> {
        Ok(self
            .session(session_id)?
            .events
            .iter()
            .filter(|event| event.sequence > after_sequence)
            .cloned()
            .collect())
    }

    fn health_check(&self, session_id: &str) -> Result<(), AgentAdapterError> {
        let session = self.session(session_id)?;
        if session.terminated {
            return Err(AgentAdapterError::CapabilityUnsupported {
                capability: "health check for a terminated session",
            });
        }
        Ok(())
    }
}
