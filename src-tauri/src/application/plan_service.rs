use crate::{
    domain::{
        Board, BoardId, Dependency, DependencyId, DependencySource, PlanId, SchemaMetadata,
        WorkItem, WorkItemId, WorkItemState,
    },
    orchestration::{DaemonScheduler, PlanConfirmation, PlanProposal},
};

use super::{
    BoardPlan, BoardRepository, BoardService, BoardServiceError, BoardSnapshot, ConfirmPlanRequest,
    ProposePlanRequest, ProposedPlanDependencyRequest, ProposedPlanWorkItemRequest,
};

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    pub fn propose_plan(
        &mut self,
        request: ProposePlanRequest,
    ) -> Result<BoardPlan, BoardServiceError<Repository::Error>> {
        validate_proposal_request(&request)?;
        let board_id = BoardId::from(request.board_id.as_str());
        let board = self
            .repository
            .board(&board_id)
            .map_err(BoardServiceError::Repository)?
            .ok_or_else(|| BoardServiceError::BoardNotFound {
                board_id: board_id.clone(),
            })?;
        let request = self.assign_default_worker(request, &board)?;
        let proposal = plan_proposal(request, &board);
        let scheduler =
            DaemonScheduler::propose(proposal.clone()).map_err(BoardServiceError::PlanProposal)?;
        self.repository
            .save_plan_proposal(proposal)
            .map_err(BoardServiceError::Repository)?;
        Ok(BoardPlan {
            preview: scheduler.preview().clone(),
            confirmation: None,
        })
    }

    pub fn board_plan(
        &self,
        board_id: &str,
    ) -> Result<Option<BoardPlan>, BoardServiceError<Repository::Error>> {
        super::board_service::validate_required(board_id, "board id")?;
        let Some(stored_plan) = self
            .repository
            .stored_plan_for_board(&BoardId::from(board_id))
            .map_err(BoardServiceError::Repository)?
        else {
            return Ok(None);
        };
        let scheduler = DaemonScheduler::propose(stored_plan.proposal)
            .map_err(BoardServiceError::PlanProposal)?;
        Ok(Some(BoardPlan {
            preview: scheduler.preview().clone(),
            confirmation: stored_plan.confirmation,
        }))
    }

    pub fn confirm_plan(
        &mut self,
        request: ConfirmPlanRequest,
    ) -> Result<BoardSnapshot, BoardServiceError<Repository::Error>> {
        super::board_service::validate_required(&request.board_id, "board id")?;
        super::board_service::validate_required(&request.plan_id, "plan id")?;
        super::board_service::validate_required(&request.confirmed_by, "plan confirmer")?;
        super::board_service::validate_required(&request.confirmed_at, "plan confirmation time")?;

        let board_id = BoardId::from(request.board_id.as_str());
        let stored_plan = self
            .repository
            .stored_plan_for_board(&board_id)
            .map_err(BoardServiceError::Repository)?
            .ok_or_else(|| BoardServiceError::PlanNotFound {
                plan_id: PlanId::from(request.plan_id.as_str()),
            })?;
        let mut scheduler = DaemonScheduler::propose(stored_plan.proposal.clone())
            .map_err(BoardServiceError::PlanProposal)?;
        let confirmation = PlanConfirmation {
            plan_id: PlanId::from(request.plan_id.as_str()),
            confirmed_by: request.confirmed_by,
            confirmed_at: request.confirmed_at,
        };
        scheduler
            .confirm(confirmation.clone())
            .map_err(BoardServiceError::PlanConfirmation)?;
        self.repository
            .confirm_and_materialize_plan(stored_plan.proposal, confirmation)
            .map_err(BoardServiceError::Repository)?;
        self.snapshot(&board_id)
    }
}

impl<Repository> BoardService<Repository>
where
    Repository: BoardRepository,
{
    fn assign_default_worker(
        &self,
        mut request: ProposePlanRequest,
        board: &Board,
    ) -> Result<ProposePlanRequest, BoardServiceError<Repository::Error>> {
        let default_worker = self
            .repository
            .project_agent_settings(&board.project_id)
            .map_err(BoardServiceError::Repository)?
            .and_then(|settings| settings.ticket_worker);
        for work_item in &mut request.work_items {
            if work_item.assigned_agent_profile_name.is_none() {
                work_item.assigned_agent_profile_name = default_worker
                    .as_ref()
                    .map(|defaults| defaults.agent_profile_name.clone());
            }
            if work_item.assigned_agent_model.is_none() {
                work_item.assigned_agent_model = default_worker
                    .as_ref()
                    .map(|defaults| defaults.model.clone());
            }
            if work_item.assigned_agent_effort.is_none() {
                work_item.assigned_agent_effort =
                    default_worker.as_ref().map(|defaults| defaults.effort);
            }
            if let Some(profile_name) = &work_item.assigned_agent_profile_name {
                let exists = self
                    .repository
                    .agent_profile(profile_name)
                    .map_err(BoardServiceError::Repository)?
                    .is_some();
                if !exists {
                    return Err(BoardServiceError::AgentProfileNotFound {
                        profile_name: profile_name.clone(),
                    });
                }
            }
        }
        Ok(request)
    }
}

fn validate_proposal_request<RepositoryError>(
    request: &ProposePlanRequest,
) -> Result<(), BoardServiceError<RepositoryError>> {
    super::board_service::validate_required(&request.plan_id, "plan id")?;
    super::board_service::validate_required(&request.board_id, "board id")?;
    super::board_service::validate_required(&request.proposed_by, "plan proposer")?;
    super::board_service::validate_required(&request.proposed_at, "plan proposal time")?;
    for work_item in &request.work_items {
        validate_proposed_work_item(work_item)?;
    }
    for dependency in &request.dependencies {
        validate_proposed_dependency(dependency)?;
    }
    Ok(())
}

fn validate_proposed_work_item<RepositoryError>(
    work_item: &ProposedPlanWorkItemRequest,
) -> Result<(), BoardServiceError<RepositoryError>> {
    super::board_service::validate_required(&work_item.work_item_id, "plan work item id")?;
    super::board_service::validate_required(&work_item.title, "plan work item title")?;
    super::board_service::validate_required(&work_item.description, "plan work item description")?;
    super::board_service::validate_criteria(&work_item.acceptance_criteria)
}

fn validate_proposed_dependency<RepositoryError>(
    dependency: &ProposedPlanDependencyRequest,
) -> Result<(), BoardServiceError<RepositoryError>> {
    super::board_service::validate_required(&dependency.dependency_id, "plan dependency id")?;
    super::board_service::validate_required(
        &dependency.upstream_work_item_id,
        "plan upstream work item id",
    )?;
    super::board_service::validate_required(
        &dependency.downstream_work_item_id,
        "plan downstream work item id",
    )?;
    super::board_service::validate_required(&dependency.reason, "plan dependency reason")?;
    super::board_service::validate_required(&dependency.owner, "plan dependency owner")?;
    super::board_service::validate_required(&dependency.next_action, "plan dependency next action")
}

fn plan_proposal(request: ProposePlanRequest, board: &Board) -> PlanProposal {
    PlanProposal {
        id: PlanId::from(request.plan_id.as_str()),
        project_id: board.project_id.clone(),
        work_items: request
            .work_items
            .into_iter()
            .map(|work_item| WorkItem {
                schema: SchemaMetadata::current(),
                id: WorkItemId::from(work_item.work_item_id.as_str()),
                board_id: board.id.clone(),
                title: work_item.title,
                description: work_item.description,
                acceptance_criteria: work_item.acceptance_criteria,
                budget: work_item.budget,
                state: WorkItemState::Inbox,
                requires_human_review: work_item.requires_human_review,
                assigned_agent_profile_name: work_item.assigned_agent_profile_name,
                assigned_agent_model: work_item.assigned_agent_model.unwrap_or_default(),
                assigned_agent_effort: work_item.assigned_agent_effort.unwrap_or_default(),
            })
            .collect(),
        dependencies: request
            .dependencies
            .into_iter()
            .map(|dependency| Dependency {
                schema: SchemaMetadata::current(),
                id: DependencyId::from(dependency.dependency_id.as_str()),
                upstream_work_item_id: WorkItemId::from(dependency.upstream_work_item_id.as_str()),
                downstream_work_item_id: WorkItemId::from(
                    dependency.downstream_work_item_id.as_str(),
                ),
                kind: dependency.kind,
                source: DependencySource::Orchestrator,
                reason: dependency.reason,
                owner: dependency.owner,
                next_action: dependency.next_action,
                created_by: request.proposed_by.clone(),
                created_at: request.proposed_at.clone(),
            })
            .collect(),
        unresolved_assumptions: request.unresolved_assumptions,
    }
}
