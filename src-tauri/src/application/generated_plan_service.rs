use std::collections::BTreeMap;

use chrono::Utc;
use uuid::Uuid;

use crate::orchestration::{PlanDraft, PlanDraftError};

use super::{
    GeneratePlanRequest, ProposePlanRequest, ProposedPlanDependencyRequest,
    ProposedPlanWorkItemRequest,
};

pub fn generated_plan_request(
    request: &GeneratePlanRequest,
    planner_profile_name: &str,
    draft: PlanDraft,
) -> Result<ProposePlanRequest, PlanDraftError> {
    draft.validate()?;
    let plan_id = format!("plan-{}", Uuid::new_v4());
    let work_item_ids = draft
        .work_items
        .iter()
        .enumerate()
        .map(|(index, work_item)| {
            (
                work_item.key.clone(),
                format!("{plan_id}-task-{}", index + 1),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let work_items = draft
        .work_items
        .into_iter()
        .map(|work_item| ProposedPlanWorkItemRequest {
            work_item_id: work_item_ids[&work_item.key].clone(),
            title: work_item.title,
            description: work_item.description,
            acceptance_criteria: work_item.acceptance_criteria,
            budget: work_item.budget.into(),
            requires_human_review: work_item.requires_human_review,
        })
        .collect();
    let dependencies = draft
        .dependencies
        .into_iter()
        .enumerate()
        .map(|(index, dependency)| ProposedPlanDependencyRequest {
            dependency_id: format!("{plan_id}-dependency-{}", index + 1),
            upstream_work_item_id: work_item_ids[&dependency.upstream_key].clone(),
            downstream_work_item_id: work_item_ids[&dependency.downstream_key].clone(),
            kind: dependency.kind,
            reason: dependency.reason,
            owner: dependency.owner,
            next_action: dependency.next_action,
        })
        .collect();

    Ok(ProposePlanRequest {
        plan_id,
        board_id: request.board_id.clone(),
        proposed_by: format!("planner:{planner_profile_name}"),
        proposed_at: Utc::now().to_rfc3339(),
        work_items,
        dependencies,
        unresolved_assumptions: draft.unresolved_assumptions,
    })
}
