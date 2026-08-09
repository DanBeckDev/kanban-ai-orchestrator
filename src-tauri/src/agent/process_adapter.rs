use std::{
    collections::BTreeMap,
    io::Write,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use super::process_event_reader::{
    ProcessEventProtocol, has_terminal_event, kill_child, read_events,
};
use super::{
    AgentAdapter, AgentAdapterError, AgentCapabilities, AgentProfile, AgentSession,
    NormalizedAgentEvent, StartAgentRequest,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessAgentDefinition {
    pub name: String,
    pub program: String,
    pub arguments: Vec<String>,
}

pub struct ProcessAgentAdapter {
    definition: ProcessAgentDefinition,
    protocol: ProcessEventProtocol,
    next_session_number: u64,
    sessions: BTreeMap<String, ProcessSession>,
}

struct ProcessSession {
    events: Arc<Mutex<Vec<NormalizedAgentEvent>>>,
    child: Arc<Mutex<Child>>,
}

impl ProcessAgentAdapter {
    pub(crate) fn from_structured_profile_for_execution(
        profile: AgentProfile,
        execution_id: &str,
    ) -> Self {
        Self::new(ProcessAgentDefinition {
            name: format!("{}-{execution_id}", profile.name),
            program: profile.program,
            arguments: profile.arguments,
        })
    }

    pub fn new(definition: ProcessAgentDefinition) -> Self {
        Self::new_with_event_protocol(definition, ProcessEventProtocol::Normalized)
    }

    pub(crate) fn new_with_event_protocol(
        definition: ProcessAgentDefinition,
        protocol: ProcessEventProtocol,
    ) -> Self {
        Self {
            definition,
            protocol,
            next_session_number: 1,
            sessions: BTreeMap::new(),
        }
    }

    fn session(&self, session_id: &str) -> Result<&ProcessSession, AgentAdapterError> {
        self.sessions
            .get(session_id)
            .ok_or_else(|| AgentAdapterError::UnknownSession {
                session_id: session_id.to_owned(),
            })
    }

    fn launch_process(
        &self,
        session_id: &str,
        request: &StartAgentRequest,
    ) -> Result<Child, AgentAdapterError> {
        let mut child = Command::new(&self.definition.program)
            .args(&self.definition.arguments)
            .current_dir(&request.workspace_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| AgentAdapterError::ProcessLaunch {
                adapter_name: self.definition.name.clone(),
                reason: error.to_string(),
            })?;
        let Some(mut stdin) = child.stdin.take() else {
            return Err(AgentAdapterError::ProcessInput {
                session_id: session_id.to_owned(),
                reason: "the process did not expose standard input".to_owned(),
            });
        };
        if let Err(error) = stdin
            .write_all(request.task_brief.as_bytes())
            .and_then(|()| stdin.write_all(b"\n"))
        {
            let _ = child.kill();
            return Err(AgentAdapterError::ProcessInput {
                session_id: session_id.to_owned(),
                reason: error.to_string(),
            });
        }
        Ok(child)
    }

    fn spawn_reader(
        &self,
        session_id: String,
        child: Arc<Mutex<Child>>,
        events: Arc<Mutex<Vec<NormalizedAgentEvent>>>,
    ) -> Result<(), AgentAdapterError> {
        let stdout = child
            .lock()
            .map_err(|_| AgentAdapterError::ProcessRuntime {
                session_id: session_id.clone(),
                operation: "read its event stream",
                reason: "the process state lock is unavailable".to_owned(),
            })?
            .stdout
            .take();
        let Some(stdout) = stdout else {
            kill_child(&child);
            return Err(AgentAdapterError::ProcessRuntime {
                session_id,
                operation: "read its event stream",
                reason: "the process did not expose standard output".to_owned(),
            });
        };
        let reader_session_id = session_id.clone();
        let reader_child = child.clone();
        let protocol = self.protocol;
        if let Err(error) = thread::Builder::new()
            .name(format!("agent-events-{session_id}"))
            .spawn(move || {
                read_events(stdout, &reader_session_id, &reader_child, &events, protocol)
            })
        {
            kill_child(&child);
            return Err(AgentAdapterError::ProcessRuntime {
                session_id,
                operation: "start its event reader",
                reason: error.to_string(),
            });
        }
        Ok(())
    }
}

impl AgentAdapter for ProcessAgentAdapter {
    fn name(&self) -> &str {
        &self.definition.name
    }

    fn discover(&self) -> Result<AgentCapabilities, AgentAdapterError> {
        Ok(AgentCapabilities {
            supports_feedback: false,
            supports_interrupt: false,
            supports_resume: false,
            streams_structured_events: true,
        })
    }

    fn start(&mut self, request: StartAgentRequest) -> Result<AgentSession, AgentAdapterError> {
        let session_id = format!("{}-{}", self.definition.name, self.next_session_number);
        self.next_session_number = self.next_session_number.saturating_add(1);
        let child = self.launch_process(&session_id, &request)?;
        let child = Arc::new(Mutex::new(child));
        let events = Arc::new(Mutex::new(Vec::new()));
        self.spawn_reader(session_id.clone(), child.clone(), events.clone())?;
        self.sessions
            .insert(session_id.clone(), ProcessSession { events, child });
        Ok(AgentSession {
            id: session_id,
            resumable: false,
        })
    }

    fn resume(&mut self, _session_id: &str) -> Result<AgentSession, AgentAdapterError> {
        Err(AgentAdapterError::CapabilityUnsupported {
            capability: "resume",
        })
    }

    fn send_feedback(
        &mut self,
        _session_id: &str,
        _feedback: &str,
    ) -> Result<(), AgentAdapterError> {
        Err(AgentAdapterError::CapabilityUnsupported {
            capability: "feedback",
        })
    }

    fn interrupt(&mut self, session_id: &str) -> Result<(), AgentAdapterError> {
        self.session(session_id)?;
        Err(AgentAdapterError::CapabilityUnsupported {
            capability: "process-tree interruption",
        })
    }

    fn terminate(&mut self, session_id: &str) -> Result<(), AgentAdapterError> {
        let session = self.session(session_id)?;
        session
            .child
            .lock()
            .map_err(|_| AgentAdapterError::ProcessRuntime {
                session_id: session_id.to_owned(),
                operation: "terminate the direct process",
                reason: "the process state lock is unavailable".to_owned(),
            })?
            .kill()
            .map_err(|error| AgentAdapterError::ProcessRuntime {
                session_id: session_id.to_owned(),
                operation: "terminate the direct process",
                reason: error.to_string(),
            })
    }

    fn stream_events(
        &self,
        session_id: &str,
        after_sequence: u64,
    ) -> Result<Vec<NormalizedAgentEvent>, AgentAdapterError> {
        let session = self.session(session_id)?;
        Ok(session
            .events
            .lock()
            .map_err(|_| AgentAdapterError::ProcessRuntime {
                session_id: session_id.to_owned(),
                operation: "read buffered events",
                reason: "the event queue lock is unavailable".to_owned(),
            })?
            .iter()
            .filter(|event| event.sequence > after_sequence)
            .cloned()
            .collect())
    }

    fn health_check(&self, session_id: &str) -> Result<(), AgentAdapterError> {
        let session = self.session(session_id)?;
        let status = session
            .child
            .lock()
            .map_err(|_| AgentAdapterError::ProcessRuntime {
                session_id: session_id.to_owned(),
                operation: "check process health",
                reason: "the process state lock is unavailable".to_owned(),
            })?
            .try_wait()
            .map_err(|error| AgentAdapterError::ProcessRuntime {
                session_id: session_id.to_owned(),
                operation: "check process health",
                reason: error.to_string(),
            })?;
        if status.is_none() || has_terminal_event(&session.events) {
            Ok(())
        } else {
            Err(AgentAdapterError::ProcessExited {
                session_id: session_id.to_owned(),
                exit_code: status.and_then(|status| status.code()),
            })
        }
    }
}
