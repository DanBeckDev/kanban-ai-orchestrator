use crate::{
    domain::{DependencyId, DependencyKind, PlanId, ProjectId, WorkItemBudget},
    orchestration::{DaemonScheduler, PlanConfirmation, PlanConfirmationError, PlanProposalError},
};

use super::{dependency, fully_budgeted_work_item, proposal, work_item};

#[test]
fn proposal_exposes_the_full_confirmable_execution_plan() {
    let scheduler = DaemonScheduler::propose(proposal(
        vec![
            fully_budgeted_work_item("foundation"),
            fully_budgeted_work_item("api"),
            fully_budgeted_work_item("ui"),
            work_item(
                "release",
                WorkItemBudget {
                    max_agent_turns: None,
                    max_duration_seconds: Some(90),
                    max_cost_micros: None,
                },
            ),
        ],
        vec![
            dependency(
                "foundation-api",
                "foundation",
                "api",
                DependencyKind::Blocks,
            ),
            dependency("foundation-ui", "foundation", "ui", DependencyKind::Blocks),
            dependency(
                "api-release",
                "api",
                "release",
                DependencyKind::ReviewRequired,
            ),
            dependency("ui-release-note", "ui", "release", DependencyKind::Soft),
        ],
    ))
    .expect("proposal should be valid");

    assert_eq!(
        scheduler
            .preview()
            .work_items
            .iter()
            .map(|work_item| work_item.id.clone())
            .collect::<Vec<_>>(),
        vec![
            "api".into(),
            "foundation".into(),
            "release".into(),
            "ui".into()
        ]
    );
    assert_eq!(
        scheduler.preview().critical_path,
        vec!["foundation".into(), "api".into(), "release".into()]
    );
    assert_eq!(
        scheduler
            .preview()
            .dependencies
            .iter()
            .map(|dependency| (dependency.id.clone(), dependency.kind))
            .collect::<Vec<_>>(),
        vec![
            (
                DependencyId::from("api-release"),
                DependencyKind::ReviewRequired
            ),
            (DependencyId::from("foundation-api"), DependencyKind::Blocks),
            (DependencyId::from("foundation-ui"), DependencyKind::Blocks),
            (DependencyId::from("ui-release-note"), DependencyKind::Soft),
        ]
    );
    assert_eq!(
        scheduler.preview().work_items[0].acceptance_criteria,
        vec!["api behavior is verified"]
    );
    assert_eq!(
        scheduler.preview().parallel_stages,
        vec![
            vec!["foundation".into()],
            vec!["api".into(), "ui".into()],
            vec!["release".into()]
        ]
    );
    assert_eq!(scheduler.preview().budget.max_agent_turns, None);
    assert_eq!(scheduler.preview().budget.max_duration_seconds, Some(270));
    assert_eq!(scheduler.preview().budget.max_cost_micros, None);
    assert_eq!(
        scheduler
            .preview()
            .budget
            .work_items_missing_agent_turn_budget,
        vec!["release".into()]
    );
    assert_eq!(
        scheduler.preview().budget.work_items_missing_cost_budget,
        vec!["release".into()]
    );
    assert_eq!(
        scheduler.preview().unresolved_assumptions,
        vec!["The repository default branch is available locally."]
    );
}

#[test]
fn confirmation_rejects_a_different_plan() {
    let mut scheduler =
        DaemonScheduler::propose(proposal(vec![fully_budgeted_work_item("task")], vec![]))
            .expect("proposal should be valid");

    let wrong_plan = scheduler.confirm(PlanConfirmation {
        plan_id: PlanId::from("other-plan"),
        confirmed_by: "Daniel".to_owned(),
        confirmed_at: "2026-08-08T15:01:00Z".to_owned(),
    });

    assert!(matches!(
        wrong_plan,
        Err(PlanConfirmationError::PlanIdMismatch { .. })
    ));
    assert!(scheduler.confirmation().is_none());
}

#[test]
fn confirmation_requires_a_user_identity() {
    let mut scheduler =
        DaemonScheduler::propose(proposal(vec![fully_budgeted_work_item("task")], vec![]))
            .expect("proposal should be valid");

    let blank_user = scheduler.confirm(PlanConfirmation {
        plan_id: PlanId::from("plan-1"),
        confirmed_by: " ".to_owned(),
        confirmed_at: "2026-08-08T15:01:00Z".to_owned(),
    });

    assert!(matches!(
        blank_user,
        Err(PlanConfirmationError::MissingConfirmedBy)
    ));
    assert!(scheduler.confirmation().is_none());
}

#[test]
fn confirmation_requires_a_timestamp() {
    let mut scheduler =
        DaemonScheduler::propose(proposal(vec![fully_budgeted_work_item("task")], vec![]))
            .expect("proposal should be valid");

    let blank_time = scheduler.confirm(PlanConfirmation {
        plan_id: PlanId::from("plan-1"),
        confirmed_by: "Daniel".to_owned(),
        confirmed_at: String::new(),
    });

    assert!(matches!(
        blank_time,
        Err(PlanConfirmationError::MissingConfirmedAt)
    ));
    assert!(scheduler.confirmation().is_none());
}

#[test]
fn confirmation_uses_the_application_boundary_field_names() {
    let confirmation = PlanConfirmation {
        plan_id: PlanId::from("plan-1"),
        confirmed_by: "Daniel".to_owned(),
        confirmed_at: "2026-08-08T15:01:00Z".to_owned(),
    };

    let serialized = serde_json::to_value(confirmation).expect("confirmation should serialize");

    assert_eq!(serialized["planId"], "plan-1");
    assert_eq!(serialized["confirmedBy"], "Daniel");
    assert_eq!(serialized["confirmedAt"], "2026-08-08T15:01:00Z");
}

#[test]
fn proposal_requires_at_least_one_work_item() {
    assert!(matches!(
        DaemonScheduler::propose(proposal(vec![], vec![])),
        Err(PlanProposalError::EmptyPlan)
    ));
}

#[test]
fn proposal_rejects_duplicate_work_item_ids() {
    assert!(matches!(
        DaemonScheduler::propose(proposal(
            vec![
                fully_budgeted_work_item("same"),
                fully_budgeted_work_item("same")
            ],
            vec![],
        )),
        Err(PlanProposalError::DuplicateWorkItemId { .. })
    ));
}

#[test]
fn proposal_rejects_blank_unresolved_assumptions() {
    let mut invalid_assumption = proposal(vec![fully_budgeted_work_item("task")], vec![]);
    invalid_assumption.unresolved_assumptions = vec!["\t".to_owned()];

    assert!(matches!(
        DaemonScheduler::propose(invalid_assumption),
        Err(PlanProposalError::BlankUnresolvedAssumption)
    ));
}

#[test]
fn proposal_requires_a_nonblank_plan_id() {
    let mut missing_plan_id = proposal(vec![fully_budgeted_work_item("task")], vec![]);
    missing_plan_id.id = PlanId::from(" ");

    assert!(matches!(
        DaemonScheduler::propose(missing_plan_id),
        Err(PlanProposalError::MissingPlanId)
    ));
}

#[test]
fn proposal_requires_a_nonblank_project_id() {
    let mut missing_project_id = proposal(vec![fully_budgeted_work_item("task")], vec![]);
    missing_project_id.project_id = ProjectId::from("");

    assert!(matches!(
        DaemonScheduler::propose(missing_project_id),
        Err(PlanProposalError::MissingProjectId)
    ));
}
