use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    agent::{
        AgentAdapter, AgentProfile, NormalizedAgentEvent, NormalizedAgentEventKind,
        WorkerAgentAdapter,
    },
    application::{
        ExecutionEventController, RecordEvidenceRequest, RecordExecutionRequest,
        StartExecutionRequest, UpdateExecutionRequest, prepare_execution_launch,
    },
    domain::{
        EvidenceKind, EvidenceResult, Execution, ExecutionId, ExecutionStatus, SchemaMetadata,
        WorkItemId,
    },
    workspace::{DependencySharingStrategy, WorkspaceManager, WorkspaceProvisionRequest},
};

use crate::desktop::LocalBoardService;
use crate::desktop_execution_policy::authorize_execution_start;
use crate::desktop_execution_runtime_support::{
    ExecutionRuntimeError, ensure_ready, is_terminal_event, lock, timestamp, validate_start_request,
};

#[derive(Clone)]
pub(crate) struct ExecutionRuntime {
    pub(crate) service: Arc<Mutex<LocalBoardService>>,
    pub(crate) workspace_root: PathBuf,
    launch_gate: Arc<Mutex<()>>,
    pub(crate) agents: Arc<Mutex<BTreeMap<String, WorkerAgentAdapter>>>,
    pub(crate) stop_requests: Arc<Mutex<BTreeSet<String>>>,
}

impl ExecutionRuntime {
    pub(crate) fn new(service: Arc<Mutex<LocalBoardService>>, workspace_root: PathBuf) -> Self {
        Self {
            service,
            workspace_root,
            launch_gate: Arc::new(Mutex::new(())),
            agents: Arc::new(Mutex::new(BTreeMap::new())),
            stop_requests: Arc::new(Mutex::new(BTreeSet::new())),
        }
    }

    pub(crate) fn start(
        &self,
        request: StartExecutionRequest,
    ) -> Result<crate::application::BoardSnapshot, ExecutionRuntimeError> {
        validate_start_request(&request)?;
        let _launch_gate = lock(&self.launch_gate, "execution launch gate")?;
        let work_item_id = WorkItemId::from(request.work_item_id.as_str());
        let (profile, project, work_item) =
            self.launch_context(&request.agent_profile_name, &work_item_id)?;
        ensure_ready(&work_item.work_item.id, work_item.work_item.state)?;
        self.authorize_execution_start(&project, &work_item, &request.execution_id)?;

        let manager = WorkspaceManager::new(&project, &self.workspace_root)
            .map_err(ExecutionRuntimeError::Workspace)?;
        let assignment = manager
            .provision(WorkspaceProvisionRequest {
                work_item_id: work_item_id.clone(),
                dependency_sharing: DependencySharingStrategy::IsolatedInstall,
            })
            .map_err(ExecutionRuntimeError::Workspace)?;
        let pending_execution = pending_execution(&request, &assignment);
        let preparation = prepare_execution_launch(
            &pending_execution,
            &work_item,
            &manager,
            &assignment,
            &profile.name,
            &request.task_brief,
        )
        .map_err(ExecutionRuntimeError::Preflight)?;
        self.record_pending_execution(&request, &assignment)?;

        let mut adapter =
            WorkerAgentAdapter::from_profile_for_execution(profile, &request.execution_id);
        let session = match adapter.start(preparation.request().clone()) {
            Ok(session) => session,
            Err(error) => {
                self.fail_pending_execution(&request.execution_id, &error.to_string());
                return Err(ExecutionRuntimeError::Agent(error));
            }
        };
        let snapshot = match self.activate(&request.execution_id, &session.id) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                let _ = adapter.terminate(&session.id);
                self.fail_pending_execution(&request.execution_id, &error.to_string());
                return Err(error);
            }
        };
        if let Err(error) = self.register_live_agent(&request.execution_id, adapter, &session.id) {
            self.record_monitor_failure(&request.execution_id, &error.to_string());
            return Err(error);
        }
        if let Err(error) = self.spawn_monitor(request.execution_id.clone(), session.id.clone()) {
            self.stop_agent(&request.execution_id, &session.id);
            self.record_monitor_failure(&request.execution_id, &error.to_string());
            return Err(error);
        }
        Ok(snapshot)
    }

    fn launch_context(
        &self,
        profile_name: &str,
        work_item_id: &WorkItemId,
    ) -> Result<
        (
            AgentProfile,
            crate::domain::Project,
            crate::domain::MaterializedWorkItem,
        ),
        ExecutionRuntimeError,
    > {
        let service = lock(&self.service, "board service")?;
        let profile = service
            .agent_profile(profile_name)
            .map_err(ExecutionRuntimeError::Profile)?;
        let project = service
            .project_for_work_item(work_item_id)
            .map_err(ExecutionRuntimeError::Board)?;
        let work_item = service
            .work_item(work_item_id)
            .map_err(ExecutionRuntimeError::Board)?;
        Ok((profile, project, work_item))
    }

    fn record_pending_execution(
        &self,
        request: &StartExecutionRequest,
        assignment: &crate::workspace::WorkspaceAssignment,
    ) -> Result<(), ExecutionRuntimeError> {
        let mut service = lock(&self.service, "board service")?;
        let work_item = service
            .work_item(&WorkItemId::from(request.work_item_id.as_str()))
            .map_err(ExecutionRuntimeError::Board)?;
        ensure_ready(&work_item.work_item.id, work_item.work_item.state)?;
        service
            .record_execution(RecordExecutionRequest {
                execution_id: request.execution_id.clone(),
                work_item_id: request.work_item_id.clone(),
                adapter_name: request.agent_profile_name.clone(),
                workspace_path: assignment.path().display().to_string(),
            })
            .map_err(ExecutionRuntimeError::Board)?;
        Ok(())
    }

    fn authorize_execution_start(
        &self,
        project: &crate::domain::Project,
        work_item: &crate::domain::MaterializedWorkItem,
        execution_id: &str,
    ) -> Result<(), ExecutionRuntimeError> {
        authorize_execution_start(
            &mut *lock(&self.service, "board service")?,
            project,
            work_item,
            execution_id,
            timestamp(),
        )
    }

    fn activate(
        &self,
        execution_id: &str,
        session_id: &str,
    ) -> Result<crate::application::BoardSnapshot, ExecutionRuntimeError> {
        ExecutionEventController::activate(
            &mut *lock(&self.service, "board service")?,
            execution_id,
            session_id,
            &timestamp(),
        )
        .map_err(ExecutionRuntimeError::Activation)
    }

    fn register_live_agent(
        &self,
        execution_id: &str,
        mut adapter: WorkerAgentAdapter,
        session_id: &str,
    ) -> Result<(), ExecutionRuntimeError> {
        let mut agents = match lock(&self.agents, "agent runtime") {
            Ok(agents) => agents,
            Err(error) => {
                let _ = adapter.terminate(session_id);
                return Err(error);
            }
        };
        if agents.contains_key(execution_id) {
            let error = ExecutionRuntimeError::DuplicateLiveExecution {
                execution_id: execution_id.to_owned(),
                session_id: session_id.to_owned(),
            };
            drop(agents);
            let _ = adapter.terminate(session_id);
            return Err(error);
        }
        agents.insert(execution_id.to_owned(), adapter);
        Ok(())
    }

    fn spawn_monitor(
        &self,
        execution_id: String,
        session_id: String,
    ) -> Result<(), ExecutionRuntimeError> {
        let runtime = self.clone();
        thread::Builder::new()
            .name(format!("board-agent-{execution_id}"))
            .spawn(move || runtime.monitor(execution_id, session_id))
            .map(|_| ())
            .map_err(|error| ExecutionRuntimeError::MonitorSpawn(error.to_string()))
    }

    fn monitor(&self, execution_id: String, session_id: String) {
        loop {
            let execution = match self.execution(&execution_id) {
                Ok(execution) => execution,
                Err(_) => return self.stop_agent(&execution_id, &session_id),
            };
            if self.take_stop_request(&execution_id) {
                self.interrupt_execution(&execution, &session_id);
                return;
            }
            if execution.status == ExecutionStatus::AwaitingInput {
                self.record_monitor_failure(
                    &execution_id,
                    "the configured process profile cannot accept feedback after requesting input",
                );
                return self.stop_agent(&execution_id, &session_id);
            }
            if execution.status != ExecutionStatus::Running {
                return self.stop_agent(&execution_id, &session_id);
            }
            let events =
                match self.events(&execution_id, &session_id, execution.last_event_sequence) {
                    Ok(events) => events,
                    Err(error) => {
                        self.record_monitor_failure(
                            &execution_id,
                            &format!("agent stream failed: {error}"),
                        );
                        return self.stop_agent(&execution_id, &session_id);
                    }
                };
            for event in events {
                let is_terminal = is_terminal_event(&event);
                if self.record_event(&execution_id, event).is_err() {
                    self.record_monitor_failure(
                        &execution_id,
                        "agent event could not be persisted",
                    );
                    return self.stop_agent(&execution_id, &session_id);
                }
                if is_terminal {
                    self.record_review_artifacts(&execution);
                    return self.stop_agent(&execution_id, &session_id);
                }
            }
            if let Err(error) = self.health_check(&execution_id, &session_id) {
                self.record_monitor_failure(
                    &execution_id,
                    &format!("agent exited unexpectedly: {error}"),
                );
                return self.stop_agent(&execution_id, &session_id);
            }
            thread::sleep(Duration::from_millis(200));
        }
    }

    pub(crate) fn execution(&self, execution_id: &str) -> Result<Execution, ExecutionRuntimeError> {
        lock(&self.service, "board service")?
            .execution(&ExecutionId::from(execution_id))
            .map_err(ExecutionRuntimeError::Board)
    }

    fn events(
        &self,
        execution_id: &str,
        session_id: &str,
        after_sequence: u64,
    ) -> Result<Vec<NormalizedAgentEvent>, ExecutionRuntimeError> {
        lock(&self.agents, "agent runtime")?
            .get(execution_id)
            .ok_or_else(|| ExecutionRuntimeError::MissingLiveExecution {
                execution_id: execution_id.to_owned(),
            })?
            .stream_events(session_id, after_sequence)
            .map_err(ExecutionRuntimeError::Agent)
    }

    fn health_check(
        &self,
        execution_id: &str,
        session_id: &str,
    ) -> Result<(), ExecutionRuntimeError> {
        lock(&self.agents, "agent runtime")?
            .get(execution_id)
            .ok_or_else(|| ExecutionRuntimeError::MissingLiveExecution {
                execution_id: execution_id.to_owned(),
            })?
            .health_check(session_id)
            .map_err(ExecutionRuntimeError::Agent)
    }

    pub(crate) fn record_event(
        &self,
        execution_id: &str,
        event: NormalizedAgentEvent,
    ) -> Result<(), ExecutionRuntimeError> {
        ExecutionEventController::record_event(
            &mut *lock(&self.service, "board service")?,
            execution_id,
            event,
            &timestamp(),
        )
        .map(|_| ())
        .map_err(ExecutionRuntimeError::Activation)
    }

    pub(crate) fn record_monitor_failure(&self, execution_id: &str, reason: &str) {
        let Ok(execution) = self.execution(execution_id) else {
            return;
        };
        if !matches!(
            execution.status,
            ExecutionStatus::Running | ExecutionStatus::AwaitingInput
        ) {
            return;
        }
        let event = NormalizedAgentEvent {
            sequence: execution.last_event_sequence.saturating_add(1),
            kind: NormalizedAgentEventKind::Failed {
                reason: reason.to_owned(),
            },
        };
        let _ = self.record_event(execution_id, event);
    }

    fn fail_pending_execution(&self, execution_id: &str, reason: &str) {
        let Ok(mut service) = lock(&self.service, "board service") else {
            return;
        };
        let Ok(execution) = service.execution(&ExecutionId::from(execution_id)) else {
            return;
        };
        if execution.status != ExecutionStatus::Pending {
            return;
        }
        let _ = service.update_execution(UpdateExecutionRequest {
            execution_id: execution.id.0.clone(),
            status: ExecutionStatus::Failed,
            session_id: None,
            usage: execution.usage,
            last_event_sequence: execution.last_event_sequence,
        });
        let _ = service.record_evidence(RecordEvidenceRequest {
            evidence_id: format!("launch-failure-{execution_id}"),
            work_item_id: execution.work_item_id.0,
            kind: EvidenceKind::AgentReport,
            result: EvidenceResult::Failed,
            summary: reason.to_owned(),
            recorded_at: timestamp(),
        });
    }

    pub(crate) fn stop_agent(&self, execution_id: &str, session_id: &str) {
        let adapter = lock(&self.agents, "agent runtime")
            .ok()
            .and_then(|mut agents| agents.remove(execution_id));
        if let Some(mut adapter) = adapter {
            let _ = adapter.terminate(session_id);
        }
        self.clear_stop_request(execution_id);
    }
}

fn pending_execution(
    request: &StartExecutionRequest,
    assignment: &crate::workspace::WorkspaceAssignment,
) -> Execution {
    Execution {
        schema: SchemaMetadata::current(),
        id: ExecutionId::from(request.execution_id.as_str()),
        work_item_id: WorkItemId::from(request.work_item_id.as_str()),
        adapter_name: request.agent_profile_name.clone(),
        status: ExecutionStatus::Pending,
        session_id: None,
        workspace_path: assignment.path().display().to_string(),
        usage: crate::domain::ExecutionUsage {
            input_tokens: 0,
            output_tokens: 0,
            cost_micros: None,
        },
        last_event_sequence: 0,
    }
}
