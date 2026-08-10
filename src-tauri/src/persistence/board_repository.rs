use crate::domain::{
    Board, BoardId, BoardSupervision, ConnectorOutboxItem, ConnectorOutboxItemId,
    ConnectorReconciliationItem, CreateWorkItemCommand, Dependency, Evidence, Execution,
    ExternalLink, ExternalLinkId, MaterializedWorkItem, Project, ProjectAgentSettings,
    RecordedWorkItemEvent, SupervisionDecision, TransitionWorkItemCommand, WorkItemId,
};
use crate::{
    agent::AgentProfile,
    application::{BoardLibraryRecord, BoardRepository, BoardSnapshot, StoredPlan},
    orchestration::{PlanConfirmation, PlanProposal, PlannerProfile},
};

use super::{BoardStoreError, SqliteEventStore};

impl BoardRepository for SqliteEventStore {
    type Error = BoardStoreError;

    fn create_project(&mut self, project: Project) -> Result<Project, Self::Error> {
        SqliteEventStore::create_project(self, project)
    }

    fn project(
        &self,
        project_id: &crate::domain::ProjectId,
    ) -> Result<Option<Project>, Self::Error> {
        SqliteEventStore::project(self, project_id)
    }

    fn create_board(&mut self, board: Board) -> Result<Board, Self::Error> {
        SqliteEventStore::create_board(self, board)
    }

    fn create_local_board(
        &mut self,
        project: Project,
        board: Board,
        opened_at: String,
    ) -> Result<(), Self::Error> {
        SqliteEventStore::create_local_board(self, project, board, opened_at)
    }

    fn board(&self, board_id: &BoardId) -> Result<Option<Board>, Self::Error> {
        SqliteEventStore::board(self, board_id)
    }

    fn board_library_records(&self) -> Result<Vec<BoardLibraryRecord>, Self::Error> {
        SqliteEventStore::board_library_records(self)
    }

    fn record_board_opened(
        &mut self,
        board_id: &BoardId,
        opened_at: String,
    ) -> Result<(), Self::Error> {
        SqliteEventStore::record_board_opened(self, board_id, opened_at)
    }

    fn create_board_work_item(
        &mut self,
        command: CreateWorkItemCommand,
    ) -> Result<RecordedWorkItemEvent, Self::Error> {
        SqliteEventStore::create_board_work_item(self, command)
    }

    fn add_board_dependency(&mut self, dependency: Dependency) -> Result<Dependency, Self::Error> {
        SqliteEventStore::add_board_dependency(self, dependency)
    }

    fn materialized_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Option<MaterializedWorkItem>, Self::Error> {
        SqliteEventStore::materialized_work_item(self, work_item_id).map_err(BoardStoreError::from)
    }

    fn transition_work_item(
        &mut self,
        command: TransitionWorkItemCommand,
    ) -> Result<RecordedWorkItemEvent, Self::Error> {
        SqliteEventStore::transition_work_item(self, command).map_err(BoardStoreError::from)
    }

    fn record_execution(&mut self, execution: Execution) -> Result<Execution, Self::Error> {
        SqliteEventStore::record_execution(self, execution).map_err(BoardStoreError::from)
    }

    fn execution(
        &self,
        execution_id: &crate::domain::ExecutionId,
    ) -> Result<Option<Execution>, Self::Error> {
        SqliteEventStore::execution(self, execution_id).map_err(BoardStoreError::from)
    }

    fn executions_for_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Vec<Execution>, Self::Error> {
        SqliteEventStore::executions_for_work_items(self, std::slice::from_ref(work_item_id))
            .map_err(BoardStoreError::from)
    }

    fn update_execution(&mut self, execution: Execution) -> Result<Execution, Self::Error> {
        SqliteEventStore::update_execution(self, execution).map_err(BoardStoreError::from)
    }

    fn active_execution_count_for_project(
        &self,
        project_id: &crate::domain::ProjectId,
    ) -> Result<u32, Self::Error> {
        SqliteEventStore::active_execution_count_for_project(self, project_id)
    }

    fn activate_execution_and_start_work_item(
        &mut self,
        execution: Execution,
        command: TransitionWorkItemCommand,
    ) -> Result<RecordedWorkItemEvent, Self::Error> {
        SqliteEventStore::activate_execution_and_start_work_item(self, execution, command)
            .map_err(BoardStoreError::from)
    }

    fn record_evidence(&mut self, evidence: Evidence) -> Result<Evidence, Self::Error> {
        SqliteEventStore::record_evidence(self, evidence).map_err(BoardStoreError::from)
    }

    fn record_evidence_and_transition(
        &mut self,
        evidence: Evidence,
        command: TransitionWorkItemCommand,
    ) -> Result<RecordedWorkItemEvent, Self::Error> {
        SqliteEventStore::record_evidence_and_transition(self, evidence, command)
            .map_err(BoardStoreError::from)
    }

    fn record_external_link(&mut self, link: ExternalLink) -> Result<ExternalLink, Self::Error> {
        SqliteEventStore::record_external_link(self, link).map_err(BoardStoreError::from)
    }

    fn external_link(&self, link_id: &ExternalLinkId) -> Result<Option<ExternalLink>, Self::Error> {
        SqliteEventStore::external_link(self, link_id).map_err(BoardStoreError::from)
    }

    fn external_link_for_connector_resource(
        &self,
        connector_id: &str,
        external_id: &str,
    ) -> Result<Option<ExternalLink>, Self::Error> {
        SqliteEventStore::external_link_for_connector_resource(self, connector_id, external_id)
            .map_err(BoardStoreError::from)
    }

    fn external_links_for_work_items(
        &self,
        work_item_ids: &[WorkItemId],
    ) -> Result<Vec<ExternalLink>, Self::Error> {
        SqliteEventStore::external_links_for_work_items(self, work_item_ids)
            .map_err(BoardStoreError::from)
    }

    fn record_connector_outbox_item(
        &mut self,
        item: ConnectorOutboxItem,
    ) -> Result<ConnectorOutboxItem, Self::Error> {
        SqliteEventStore::record_connector_outbox_item(self, item).map_err(BoardStoreError::from)
    }

    fn claim_connector_outbox_item(
        &mut self,
        item_id: &ConnectorOutboxItemId,
    ) -> Result<ConnectorOutboxItem, Self::Error> {
        SqliteEventStore::claim_connector_outbox_item(self, item_id).map_err(BoardStoreError::from)
    }

    fn mark_connector_outbox_delivered(
        &mut self,
        item_id: &ConnectorOutboxItemId,
        delivered_at: String,
    ) -> Result<ConnectorOutboxItem, Self::Error> {
        SqliteEventStore::mark_connector_outbox_delivered(self, item_id, delivered_at)
            .map_err(BoardStoreError::from)
    }

    fn mark_connector_outbox_delivery_uncertain(
        &mut self,
        item_id: &ConnectorOutboxItemId,
    ) -> Result<ConnectorOutboxItem, Self::Error> {
        SqliteEventStore::mark_connector_outbox_delivery_uncertain(self, item_id)
            .map_err(BoardStoreError::from)
    }

    fn connector_outbox_items_for_work_items(
        &self,
        work_item_ids: &[WorkItemId],
    ) -> Result<Vec<ConnectorOutboxItem>, Self::Error> {
        SqliteEventStore::connector_outbox_items_for_work_items(self, work_item_ids)
            .map_err(BoardStoreError::from)
    }

    fn record_connector_reconciliation_item(
        &mut self,
        item: ConnectorReconciliationItem,
    ) -> Result<ConnectorReconciliationItem, Self::Error> {
        SqliteEventStore::record_connector_reconciliation_item(self, item)
            .map_err(BoardStoreError::from)
    }

    fn connector_reconciliation_items_for_work_items(
        &self,
        work_item_ids: &[WorkItemId],
    ) -> Result<Vec<ConnectorReconciliationItem>, Self::Error> {
        SqliteEventStore::connector_reconciliation_items_for_work_items(self, work_item_ids)
            .map_err(BoardStoreError::from)
    }

    fn evidence_for_work_item(
        &self,
        work_item_id: &WorkItemId,
    ) -> Result<Vec<Evidence>, Self::Error> {
        SqliteEventStore::evidence_for_work_items(self, std::slice::from_ref(work_item_id))
            .map_err(BoardStoreError::from)
    }

    fn save_agent_profile(&mut self, profile: AgentProfile) -> Result<AgentProfile, Self::Error> {
        SqliteEventStore::save_agent_profile(self, profile).map_err(BoardStoreError::from)
    }

    fn agent_profile(&self, name: &str) -> Result<Option<AgentProfile>, Self::Error> {
        SqliteEventStore::agent_profile(self, name).map_err(BoardStoreError::from)
    }

    fn agent_profiles(&self) -> Result<Vec<AgentProfile>, Self::Error> {
        SqliteEventStore::agent_profiles(self).map_err(BoardStoreError::from)
    }

    fn save_planner_profile(
        &mut self,
        profile: PlannerProfile,
    ) -> Result<PlannerProfile, Self::Error> {
        SqliteEventStore::save_planner_profile(self, profile).map_err(BoardStoreError::from)
    }

    fn planner_profile(&self, name: &str) -> Result<Option<PlannerProfile>, Self::Error> {
        SqliteEventStore::planner_profile(self, name).map_err(BoardStoreError::from)
    }

    fn planner_profiles(&self) -> Result<Vec<PlannerProfile>, Self::Error> {
        SqliteEventStore::planner_profiles(self).map_err(BoardStoreError::from)
    }

    fn save_project_agent_settings(
        &mut self,
        settings: ProjectAgentSettings,
    ) -> Result<ProjectAgentSettings, Self::Error> {
        SqliteEventStore::save_project_agent_settings(self, settings).map_err(BoardStoreError::from)
    }

    fn project_agent_settings(
        &self,
        project_id: &crate::domain::ProjectId,
    ) -> Result<Option<ProjectAgentSettings>, Self::Error> {
        SqliteEventStore::project_agent_settings(self, project_id).map_err(BoardStoreError::from)
    }

    fn save_board_supervision(
        &mut self,
        supervision: BoardSupervision,
    ) -> Result<BoardSupervision, Self::Error> {
        SqliteEventStore::save_board_supervision(self, supervision).map_err(BoardStoreError::from)
    }

    fn board_supervision(
        &self,
        board_id: &BoardId,
    ) -> Result<Option<BoardSupervision>, Self::Error> {
        SqliteEventStore::board_supervision(self, board_id).map_err(BoardStoreError::from)
    }

    fn record_supervision_decision(
        &mut self,
        decision: SupervisionDecision,
    ) -> Result<SupervisionDecision, Self::Error> {
        SqliteEventStore::record_supervision_decision(self, decision).map_err(BoardStoreError::from)
    }

    fn resolve_supervision_decision(
        &mut self,
        decision: SupervisionDecision,
    ) -> Result<SupervisionDecision, Self::Error> {
        SqliteEventStore::resolve_supervision_decision(self, decision)
            .map_err(BoardStoreError::from)
    }

    fn supervision_decisions_for_board(
        &self,
        board_id: &BoardId,
    ) -> Result<Vec<SupervisionDecision>, Self::Error> {
        SqliteEventStore::supervision_decisions_for_board(self, board_id)
            .map_err(BoardStoreError::from)
    }

    fn save_plan_proposal(&mut self, proposal: PlanProposal) -> Result<(), Self::Error> {
        SqliteEventStore::save_plan_proposal(self, proposal)
    }

    fn stored_plan_for_board(&self, board_id: &BoardId) -> Result<Option<StoredPlan>, Self::Error> {
        SqliteEventStore::stored_plan_for_board(self, board_id)
    }

    fn confirm_and_materialize_plan(
        &mut self,
        proposal: PlanProposal,
        confirmation: PlanConfirmation,
    ) -> Result<(), Self::Error> {
        SqliteEventStore::confirm_and_materialize_plan(self, proposal, confirmation)
    }

    fn board_snapshot(&self, board_id: &BoardId) -> Result<BoardSnapshot, Self::Error> {
        SqliteEventStore::board_snapshot(self, board_id)
    }
}
