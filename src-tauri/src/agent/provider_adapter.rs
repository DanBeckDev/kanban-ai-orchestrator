use super::process_event_reader::ProcessEventProtocol;
use super::{
    AgentAdapter, AgentAdapterError, AgentCapabilities, AgentProfile, AgentProfileKind,
    AgentSession, NormalizedAgentEvent, ProcessAgentAdapter, ProcessAgentDefinition,
    StartAgentRequest,
};

pub enum WorkerAgentAdapter {
    Structured(ProcessAgentAdapter),
    Native(NativeProcessAdapter),
}

impl WorkerAgentAdapter {
    pub fn from_profile_for_execution(profile: AgentProfile, execution_id: &str) -> Self {
        match profile.kind {
            AgentProfileKind::StructuredProcess => Self::Structured(
                ProcessAgentAdapter::from_structured_profile_for_execution(profile, execution_id),
            ),
            AgentProfileKind::CodexCli
            | AgentProfileKind::ClaudeCode
            | AgentProfileKind::ClinePassCli => Self::Native(
                NativeProcessAdapter::from_profile_for_execution(profile, execution_id),
            ),
        }
    }

    fn inner(&self) -> &ProcessAgentAdapter {
        match self {
            Self::Structured(adapter) => adapter,
            Self::Native(adapter) => &adapter.inner,
        }
    }

    fn inner_mut(&mut self) -> &mut ProcessAgentAdapter {
        match self {
            Self::Structured(adapter) => adapter,
            Self::Native(adapter) => &mut adapter.inner,
        }
    }

    fn capabilities(&self) -> AgentCapabilities {
        match self {
            Self::Structured(_) => AgentProfileKind::StructuredProcess.capabilities(),
            Self::Native(adapter) => adapter.kind.capabilities(),
        }
    }
}

impl AgentAdapter for WorkerAgentAdapter {
    fn name(&self) -> &str {
        self.inner().name()
    }

    fn discover(&self) -> Result<AgentCapabilities, AgentAdapterError> {
        Ok(self.capabilities())
    }

    fn start(&mut self, request: StartAgentRequest) -> Result<AgentSession, AgentAdapterError> {
        self.inner_mut().start(request)
    }

    fn resume(&mut self, session_id: &str) -> Result<AgentSession, AgentAdapterError> {
        self.inner_mut().resume(session_id)
    }

    fn send_feedback(&mut self, session_id: &str, feedback: &str) -> Result<(), AgentAdapterError> {
        self.inner_mut().send_feedback(session_id, feedback)
    }

    fn interrupt(&mut self, session_id: &str) -> Result<(), AgentAdapterError> {
        self.inner_mut().interrupt(session_id)
    }

    fn terminate(&mut self, session_id: &str) -> Result<(), AgentAdapterError> {
        self.inner_mut().terminate(session_id)
    }

    fn stream_events(
        &self,
        session_id: &str,
        after_sequence: u64,
    ) -> Result<Vec<NormalizedAgentEvent>, AgentAdapterError> {
        self.inner().stream_events(session_id, after_sequence)
    }

    fn health_check(&self, session_id: &str) -> Result<(), AgentAdapterError> {
        self.inner().health_check(session_id)
    }
}

pub struct NativeProcessAdapter {
    inner: ProcessAgentAdapter,
    kind: AgentProfileKind,
}

impl NativeProcessAdapter {
    fn from_profile_for_execution(profile: AgentProfile, execution_id: &str) -> Self {
        let kind = profile.kind;
        let definition = native_definition(profile, execution_id);
        Self::new(definition, kind)
    }

    fn new(definition: ProcessAgentDefinition, kind: AgentProfileKind) -> Self {
        Self {
            inner: ProcessAgentAdapter::new_with_event_protocol(
                definition,
                native_event_protocol(kind),
            ),
            kind,
        }
    }

    #[cfg(test)]
    pub(super) fn for_test(definition: ProcessAgentDefinition, kind: AgentProfileKind) -> Self {
        Self::new(definition, kind)
    }
}

impl AgentAdapter for NativeProcessAdapter {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn discover(&self) -> Result<AgentCapabilities, AgentAdapterError> {
        Ok(self.kind.capabilities())
    }

    fn start(&mut self, request: StartAgentRequest) -> Result<AgentSession, AgentAdapterError> {
        self.inner.start(request)
    }

    fn resume(&mut self, session_id: &str) -> Result<AgentSession, AgentAdapterError> {
        self.inner.resume(session_id)
    }

    fn send_feedback(&mut self, session_id: &str, feedback: &str) -> Result<(), AgentAdapterError> {
        self.inner.send_feedback(session_id, feedback)
    }

    fn interrupt(&mut self, session_id: &str) -> Result<(), AgentAdapterError> {
        self.inner.interrupt(session_id)
    }

    fn terminate(&mut self, session_id: &str) -> Result<(), AgentAdapterError> {
        self.inner.terminate(session_id)
    }

    fn stream_events(
        &self,
        session_id: &str,
        after_sequence: u64,
    ) -> Result<Vec<NormalizedAgentEvent>, AgentAdapterError> {
        self.inner.stream_events(session_id, after_sequence)
    }

    fn health_check(&self, session_id: &str) -> Result<(), AgentAdapterError> {
        self.inner.health_check(session_id)
    }
}

pub(super) fn native_definition(
    profile: AgentProfile,
    execution_id: &str,
) -> ProcessAgentDefinition {
    let kind = profile.kind;
    let mut arguments = native_arguments(kind);
    arguments.extend(profile.arguments);
    if kind == AgentProfileKind::CodexCli {
        arguments.push("-".to_owned());
    }
    definition(profile.name, execution_id, profile.program, arguments)
}

fn native_arguments(kind: AgentProfileKind) -> Vec<String> {
    match kind {
        AgentProfileKind::CodexCli => vec![
            "exec".to_owned(),
            "--json".to_owned(),
            "--sandbox".to_owned(),
            "workspace-write".to_owned(),
        ],
        AgentProfileKind::ClaudeCode => vec![
            "--print".to_owned(),
            "--output-format".to_owned(),
            "stream-json".to_owned(),
            "--verbose".to_owned(),
            "--permission-mode".to_owned(),
            "acceptEdits".to_owned(),
        ],
        AgentProfileKind::ClinePassCli => vec![
            "--json".to_owned(),
            "--provider".to_owned(),
            "cline".to_owned(),
            "--auto-approve".to_owned(),
            "true".to_owned(),
        ],
        AgentProfileKind::StructuredProcess => Vec::new(),
    }
}

fn native_event_protocol(kind: AgentProfileKind) -> ProcessEventProtocol {
    match kind {
        AgentProfileKind::CodexCli => ProcessEventProtocol::Codex,
        AgentProfileKind::ClaudeCode => ProcessEventProtocol::ClaudeCode,
        AgentProfileKind::ClinePassCli => ProcessEventProtocol::ClinePass,
        AgentProfileKind::StructuredProcess => ProcessEventProtocol::Normalized,
    }
}

fn definition(
    name: String,
    execution_id: &str,
    program: String,
    arguments: Vec<String>,
) -> ProcessAgentDefinition {
    ProcessAgentDefinition {
        name: format!("{name}-{execution_id}"),
        program,
        arguments,
    }
}
