use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    agent::{AgentAdapter, NormalizedAgentEvent},
    application::ExecutionEventController,
    domain::{Execution, ExecutionId, ExecutionStatus},
};

use crate::desktop::LocalBoardService;
use crate::desktop_execution_activity::{ExecutionActivityPage, ExecutionActivityStreams};
use crate::desktop_execution_runtime_support::{
    ExecutionRuntimeError, is_terminal_event, lock, timestamp,
};

#[path = "desktop_execution_runtime_launch.rs"]
mod launch;
#[path = "desktop_execution_runtime_supervision.rs"]
mod supervision;
#[path = "desktop_execution_runtime_supervision_recovery.rs"]
mod supervision_recovery;
#[path = "desktop_execution_runtime_supervision_selection.rs"]
mod supervision_selection;

#[cfg(test)]
#[path = "desktop_execution_runtime_supervision_test_fixtures.rs"]
mod supervision_test_fixtures;
#[cfg(test)]
#[path = "desktop_execution_runtime_supervision_tests.rs"]
mod supervision_tests;

type LiveAgent = Box<dyn AgentAdapter + Send>;

#[derive(Clone)]
pub(crate) struct ExecutionRuntime {
    pub(crate) service: Arc<Mutex<LocalBoardService>>,
    pub(crate) workspace_root: PathBuf,
    launch_gate: Arc<Mutex<()>>,
    pub(crate) supervision_gate: Arc<Mutex<()>>,
    pub(crate) agents: Arc<Mutex<BTreeMap<String, LiveAgent>>>,
    activity_streams: Arc<Mutex<ExecutionActivityStreams>>,
    pub(crate) stop_requests: Arc<Mutex<BTreeSet<String>>>,
}

impl ExecutionRuntime {
    pub(crate) fn new(service: Arc<Mutex<LocalBoardService>>, workspace_root: PathBuf) -> Self {
        Self {
            service,
            workspace_root,
            launch_gate: Arc::new(Mutex::new(())),
            supervision_gate: Arc::new(Mutex::new(())),
            agents: Arc::new(Mutex::new(BTreeMap::new())),
            activity_streams: Arc::new(Mutex::new(ExecutionActivityStreams::default())),
            stop_requests: Arc::new(Mutex::new(BTreeSet::new())),
        }
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
                    self.coordinate_after_execution(&execution.work_item_id);
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
        let activity_event = event.clone();
        let recorded_at = timestamp();
        ExecutionEventController::record_event(
            &mut *lock(&self.service, "board service")?,
            execution_id,
            event,
            &recorded_at,
        )
        .map_err(ExecutionRuntimeError::Activation)?;
        self.record_activity(execution_id, &activity_event, &recorded_at);
        Ok(())
    }

    pub(crate) fn activity_page(
        &self,
        execution_id: &str,
        after_sequence: Option<u64>,
    ) -> Result<ExecutionActivityPage, ExecutionRuntimeError> {
        Ok(lock(&self.activity_streams, "activity stream")?.page(execution_id, after_sequence))
    }

    pub(crate) fn stop_agent(&self, execution_id: &str, session_id: &str) {
        let adapter = lock(&self.agents, "agent runtime")
            .ok()
            .and_then(|mut agents| agents.remove(execution_id));
        if let Some(mut adapter) = adapter {
            let _ = adapter.terminate(session_id);
        }
        if let Ok(mut streams) = self.activity_streams.lock() {
            streams.complete(execution_id);
        }
        self.clear_stop_request(execution_id);
    }

    fn record_activity(&self, execution_id: &str, event: &NormalizedAgentEvent, recorded_at: &str) {
        if let Ok(mut streams) = self.activity_streams.lock() {
            streams.record(execution_id, event, recorded_at);
        }
    }
}
