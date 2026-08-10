use crate::{
    agent::{AgentProfile, WorkerAgentAdapter},
    application::{
        ExecutionEventController, RecordExecutionRequest, StartExecutionRequest,
        prepare_execution_launch,
    },
    desktop_execution_policy::authorize_execution_start,
    desktop_execution_runtime::{ExecutionRuntime, LiveAgent},
    desktop_execution_runtime_support::{
        ExecutionRuntimeError, ensure_startable, lock, timestamp, validate_start_request,
    },
    domain::{Execution, ExecutionStatus, SchemaMetadata, WorkItemId},
    workspace::{DependencySharingStrategy, WorkspaceManager, WorkspaceProvisionRequest},
};

enum MonitorMode {
    Background,
    #[cfg(test)]
    Synchronous,
}

impl ExecutionRuntime {
    pub(crate) fn start(
        &self,
        request: StartExecutionRequest,
    ) -> Result<crate::application::BoardSnapshot, ExecutionRuntimeError> {
        self.start_with_adapter(
            request,
            |profile, execution_id, model, effort| {
                Box::new(WorkerAgentAdapter::from_profile_for_execution(
                    profile,
                    execution_id,
                    model,
                    effort,
                ))
            },
            MonitorMode::Background,
        )
    }

    #[cfg(test)]
    pub(crate) fn start_with_test_adapter(
        &self,
        request: StartExecutionRequest,
        adapter: LiveAgent,
    ) -> Result<crate::application::BoardSnapshot, ExecutionRuntimeError> {
        self.start_with_adapter(request, move |_, _, _, _| adapter, MonitorMode::Synchronous)
    }

    fn start_with_adapter<F>(
        &self,
        request: StartExecutionRequest,
        adapter_factory: F,
        monitor_mode: MonitorMode,
    ) -> Result<crate::application::BoardSnapshot, ExecutionRuntimeError>
    where
        F: FnOnce(
            AgentProfile,
            &str,
            &crate::domain::AgentModelPreference,
            crate::domain::AgentEffort,
        ) -> LiveAgent,
    {
        validate_start_request(&request)?;
        let _launch_gate = lock(&self.launch_gate, "execution launch gate")?;
        let work_item_id = WorkItemId::from(request.work_item_id.as_str());
        let (profile, project, work_item) =
            self.launch_context(&request.agent_profile_name, &work_item_id)?;
        ensure_startable(
            &work_item.work_item.id,
            work_item.work_item.state,
            request.execution_role,
        )?;
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

        let mut adapter = adapter_factory(
            profile,
            &request.execution_id,
            &work_item.work_item.assigned_agent_model,
            work_item.work_item.assigned_agent_effort,
        );
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
        let monitor_result = match monitor_mode {
            MonitorMode::Background => {
                self.spawn_monitor(request.execution_id.clone(), session.id.clone())
            }
            #[cfg(test)]
            MonitorMode::Synchronous => {
                self.monitor(request.execution_id.clone(), session.id.clone());
                Ok(())
            }
        };
        if let Err(error) = monitor_result {
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
        ensure_startable(
            &work_item.work_item.id,
            work_item.work_item.state,
            request.execution_role,
        )?;
        service
            .record_execution(RecordExecutionRequest {
                execution_id: request.execution_id.clone(),
                work_item_id: request.work_item_id.clone(),
                adapter_name: request.agent_profile_name.clone(),
                workspace_path: assignment.path().display().to_string(),
                role: request.execution_role,
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
        mut adapter: LiveAgent,
        session_id: &str,
    ) -> Result<(), ExecutionRuntimeError> {
        let agents = match lock(&self.agents, "agent runtime") {
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
        drop(agents);
        let mut activity_streams = match lock(&self.activity_streams, "activity stream") {
            Ok(activity_streams) => activity_streams,
            Err(error) => {
                let _ = adapter.terminate(session_id);
                return Err(error);
            }
        };
        activity_streams.activate(execution_id);
        drop(activity_streams);
        lock(&self.agents, "agent runtime")?.insert(execution_id.to_owned(), adapter);
        Ok(())
    }
}

fn pending_execution(
    request: &StartExecutionRequest,
    assignment: &crate::workspace::WorkspaceAssignment,
) -> Execution {
    Execution {
        schema: SchemaMetadata::current(),
        id: crate::domain::ExecutionId::from(request.execution_id.as_str()),
        work_item_id: WorkItemId::from(request.work_item_id.as_str()),
        role: request.execution_role,
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
