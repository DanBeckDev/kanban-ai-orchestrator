use crate::{
    domain::DependencyKind,
    orchestration::{
        MAX_PLAN_ASSUMPTIONS, MAX_PLAN_DEPENDENCIES, MAX_PLAN_WORK_ITEMS, PlanDraft,
        PlanDraftDependency, PlanDraftError, PlanDraftWorkItem, PlannerProfile,
        PlannerProfileError,
    },
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
fn rejects_duplicate_or_unknown_work_item_keys_before_a_plan_can_be_proposed() {
    let duplicate = PlanDraft {
        work_items: vec![work_item("foundation"), work_item("foundation")],
        dependencies: Vec::new(),
        unresolved_assumptions: Vec::new(),
    };
    assert!(matches!(
        duplicate.validate(),
        Err(PlanDraftError::DuplicateWorkItemKey { key }) if key == "foundation"
    ));

    let unknown_dependency = PlanDraft {
        work_items: vec![work_item("foundation")],
        dependencies: vec![PlanDraftDependency {
            upstream_key: "foundation".to_owned(),
            downstream_key: "unknown".to_owned(),
            kind: DependencyKind::Blocks,
            reason: "The unknown task needs the foundation.".to_owned(),
            owner: "orchestrator".to_owned(),
            next_action: "Create the foundation.".to_owned(),
        }],
        unresolved_assumptions: Vec::new(),
    };
    assert!(matches!(
        unknown_dependency.validate(),
        Err(PlanDraftError::UnknownDependencyWorkItemKey { key }) if key == "unknown"
    ));
}

#[test]
fn rejects_unrecognized_model_output_fields_and_invalid_planner_profiles() {
    assert!(
        serde_json::from_str::<PlanDraft>(
            r#"{"workItems":[],"unexpected":"must not survive the boundary"}"#,
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<PlanDraft>(
            r#"{"workItems":[{"key":"foundation","title":"Foundation","description":"Create the contract.","acceptanceCriteria":["Contract tests pass."],"budget":{"unexpected":1}}]}"#,
        )
        .is_err()
    );
    assert!(
        PlannerProfile {
            name: "planner".to_owned(),
            program: " ".to_owned(),
            arguments: Vec::new(),
        }
        .validate()
        .is_err()
    );
}

#[test]
fn rejects_profile_nulls_and_draft_limits_before_the_board_boundary() {
    let name_with_null = PlannerProfile {
        name: "planner\0profile".to_owned(),
        program: "planner-bridge".to_owned(),
        arguments: Vec::new(),
    };
    assert!(matches!(
        name_with_null.validate(),
        Err(PlannerProfileError::FieldContainsNull {
            field: "planner profile name"
        })
    ));
    let argument_with_null = PlannerProfile {
        name: "planner".to_owned(),
        program: "planner-bridge".to_owned(),
        arguments: vec!["--model\0unsafe".to_owned()],
    };
    assert!(matches!(
        argument_with_null.validate(),
        Err(PlannerProfileError::ArgumentContainsNull)
    ));

    let too_many_work_items = PlanDraft {
        work_items: (0..=MAX_PLAN_WORK_ITEMS)
            .map(|index| work_item(&format!("task-{index}")))
            .collect(),
        dependencies: Vec::new(),
        unresolved_assumptions: Vec::new(),
    };
    assert!(matches!(
        too_many_work_items.validate(),
        Err(PlanDraftError::TooManyWorkItems)
    ));
    let too_many_dependencies = PlanDraft {
        work_items: vec![work_item("foundation")],
        dependencies: (0..=MAX_PLAN_DEPENDENCIES)
            .map(|_| dependency("foundation", "foundation"))
            .collect(),
        unresolved_assumptions: Vec::new(),
    };
    assert!(matches!(
        too_many_dependencies.validate(),
        Err(PlanDraftError::TooManyDependencies)
    ));
    let too_many_assumptions = PlanDraft {
        work_items: vec![work_item("foundation")],
        dependencies: Vec::new(),
        unresolved_assumptions: (0..=MAX_PLAN_ASSUMPTIONS)
            .map(|index| format!("assumption {index}"))
            .collect(),
    };
    assert!(matches!(
        too_many_assumptions.validate(),
        Err(PlanDraftError::TooManyAssumptions)
    ));
}

#[test]
fn rejects_blank_draft_facts_that_would_make_a_plan_ambiguous() {
    let blank_criterion = PlanDraft {
        work_items: vec![PlanDraftWorkItem {
            acceptance_criteria: vec![" ".to_owned()],
            ..work_item("foundation")
        }],
        dependencies: Vec::new(),
        unresolved_assumptions: Vec::new(),
    };
    assert!(matches!(
        blank_criterion.validate(),
        Err(PlanDraftError::InvalidAcceptanceCriteria { key }) if key == "foundation"
    ));
    let blank_assumption = PlanDraft {
        work_items: vec![work_item("foundation")],
        dependencies: Vec::new(),
        unresolved_assumptions: vec![" ".to_owned()],
    };
    assert!(matches!(
        blank_assumption.validate(),
        Err(PlanDraftError::BlankAssumption)
    ));
    let blank_dependency_key = PlanDraft {
        work_items: vec![work_item("foundation")],
        dependencies: vec![dependency(" ", "foundation")],
        unresolved_assumptions: Vec::new(),
    };
    assert!(matches!(
        blank_dependency_key.validate(),
        Err(PlanDraftError::MissingRequiredField {
            field: "dependency upstream key"
        })
    ));
}

fn dependency(upstream_key: &str, downstream_key: &str) -> PlanDraftDependency {
    PlanDraftDependency {
        upstream_key: upstream_key.to_owned(),
        downstream_key: downstream_key.to_owned(),
        kind: DependencyKind::Blocks,
        reason: "The downstream task needs the upstream task.".to_owned(),
        owner: "orchestrator".to_owned(),
        next_action: "Complete the upstream task.".to_owned(),
    }
}
