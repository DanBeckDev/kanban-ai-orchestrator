use crate::{
    application::{GeneratePlanRequest, generated_plan_request},
    domain::DependencyKind,
    orchestration::{PlanDraft, PlanDraftDependency, PlanDraftWorkItem},
};

fn work_item(key: &str) -> PlanDraftWorkItem {
    PlanDraftWorkItem {
        key: key.to_owned(),
        title: format!("{key} title"),
        description: format!("{key} description"),
        acceptance_criteria: vec![format!("{key} passes")],
        budget: Default::default(),
        requires_human_review: true,
    }
}

#[test]
fn daemon_derives_plan_identity_and_task_identifiers_from_a_safe_draft() {
    let request = GeneratePlanRequest {
        board_id: "board-1".to_owned(),
        planner_profile_name: "local planner".to_owned(),
        goal: "Do not persist this raw goal with the plan output.".to_owned(),
    };
    let proposal = generated_plan_request(
        &request,
        "local planner",
        PlanDraft {
            work_items: vec![work_item("foundation"), work_item("interface")],
            dependencies: vec![PlanDraftDependency {
                upstream_key: "foundation".to_owned(),
                downstream_key: "interface".to_owned(),
                kind: DependencyKind::Blocks,
                reason: "The interface needs the foundation.".to_owned(),
                owner: "orchestrator".to_owned(),
                next_action: "Finish the foundation.".to_owned(),
            }],
            unresolved_assumptions: vec!["The declared repository is available.".to_owned()],
        },
    )
    .expect("draft should be valid");

    assert_eq!(proposal.board_id, "board-1");
    assert_eq!(proposal.proposed_by, "planner:local planner");
    assert!(proposal.plan_id.starts_with("plan-"));
    assert!(
        proposal
            .work_items
            .iter()
            .all(|item| item.work_item_id.starts_with(&proposal.plan_id))
    );
    assert_eq!(proposal.dependencies.len(), 1);
    assert_eq!(
        proposal.dependencies[0].upstream_work_item_id,
        proposal.work_items[0].work_item_id
    );
    assert_eq!(
        proposal.dependencies[0].downstream_work_item_id,
        proposal.work_items[1].work_item_id
    );
    assert!(
        proposal
            .work_items
            .iter()
            .all(|item| !item.description.contains(&request.goal))
    );
}
