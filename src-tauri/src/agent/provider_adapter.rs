use super::process_event_reader::ProcessEventProtocol;
use super::{
    AgentAdapter, AgentAdapterError, AgentCapabilities, AgentProfile, AgentProfileKind,
    AgentSession, NormalizedAgentEvent, ProcessAgentAdapter, ProcessAgentDefinition,
    StartAgentRequest,
};

pub enum WorkerAgentAdapter {
    Structured(ProcessAgentAdapter),
    Codex(CodexCliAdapter),
    ClaudeCode(ClaudeCodeAdapter),
}

impl WorkerAgentAdapter {
    pub fn from_profile_for_execution(profile: AgentProfile, execution_id: &str) -> Self {
        match profile.kind {
            AgentProfileKind::StructuredProcess => Self::Structured(
                ProcessAgentAdapter::from_structured_profile_for_execution(profile, execution_id),
            ),
            AgentProfileKind::CodexCli => Self::Codex(CodexCliAdapter::from_profile_for_execution(
                profile,
                execution_id,
            )),
            AgentProfileKind::ClaudeCode => Self::ClaudeCode(
                ClaudeCodeAdapter::from_profile_for_execution(profile, execution_id),
            ),
        }
    }

    fn inner(&self) -> &ProcessAgentAdapter {
        match self {
            Self::Structured(adapter) => adapter,
            Self::Codex(adapter) => &adapter.inner,
            Self::ClaudeCode(adapter) => &adapter.inner,
        }
    }

    fn inner_mut(&mut self) -> &mut ProcessAgentAdapter {
        match self {
            Self::Structured(adapter) => adapter,
            Self::Codex(adapter) => &mut adapter.inner,
            Self::ClaudeCode(adapter) => &mut adapter.inner,
        }
    }

    fn capabilities(&self) -> AgentCapabilities {
        match self {
            Self::Structured(_) => AgentProfileKind::StructuredProcess.capabilities(),
            Self::Codex(_) => AgentProfileKind::CodexCli.capabilities(),
            Self::ClaudeCode(_) => AgentProfileKind::ClaudeCode.capabilities(),
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

pub struct CodexCliAdapter {
    inner: ProcessAgentAdapter,
}

impl CodexCliAdapter {
    pub(crate) fn from_profile_for_execution(profile: AgentProfile, execution_id: &str) -> Self {
        Self {
            inner: ProcessAgentAdapter::new_with_event_protocol(
                codex_definition(profile, execution_id),
                ProcessEventProtocol::Codex,
            ),
        }
    }

    #[cfg(test)]
    pub(super) fn for_test(definition: ProcessAgentDefinition) -> Self {
        Self {
            inner: ProcessAgentAdapter::new_with_event_protocol(
                definition,
                ProcessEventProtocol::Codex,
            ),
        }
    }
}

impl AgentAdapter for CodexCliAdapter {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn discover(&self) -> Result<AgentCapabilities, AgentAdapterError> {
        Ok(AgentProfileKind::CodexCli.capabilities())
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

pub struct ClaudeCodeAdapter {
    inner: ProcessAgentAdapter,
}

impl ClaudeCodeAdapter {
    pub(crate) fn from_profile_for_execution(profile: AgentProfile, execution_id: &str) -> Self {
        Self {
            inner: ProcessAgentAdapter::new_with_event_protocol(
                claude_code_definition(profile, execution_id),
                ProcessEventProtocol::ClaudeCode,
            ),
        }
    }

    #[cfg(test)]
    pub(super) fn for_test(definition: ProcessAgentDefinition) -> Self {
        Self {
            inner: ProcessAgentAdapter::new_with_event_protocol(
                definition,
                ProcessEventProtocol::ClaudeCode,
            ),
        }
    }
}

impl AgentAdapter for ClaudeCodeAdapter {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn discover(&self) -> Result<AgentCapabilities, AgentAdapterError> {
        Ok(AgentProfileKind::ClaudeCode.capabilities())
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

pub(super) fn codex_definition(
    profile: AgentProfile,
    execution_id: &str,
) -> ProcessAgentDefinition {
    let mut arguments = vec![
        "exec".to_owned(),
        "--json".to_owned(),
        "--sandbox".to_owned(),
        "workspace-write".to_owned(),
    ];
    arguments.extend(profile.arguments);
    arguments.push("-".to_owned());
    definition(profile.name, execution_id, profile.program, arguments)
}

pub(super) fn claude_code_definition(
    profile: AgentProfile,
    execution_id: &str,
) -> ProcessAgentDefinition {
    let mut arguments = vec![
        "--print".to_owned(),
        "--output-format".to_owned(),
        "stream-json".to_owned(),
        "--verbose".to_owned(),
        "--permission-mode".to_owned(),
        "acceptEdits".to_owned(),
    ];
    arguments.extend(profile.arguments);
    definition(profile.name, execution_id, profile.program, arguments)
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
