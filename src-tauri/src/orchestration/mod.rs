mod bounded_process;
mod plan;
mod planner;
mod planner_process;
mod scheduler;
mod supervisor;
mod supervisor_process;
mod ticket_effect;
mod ticket_effect_process;

pub use plan::{
    PlanBudgetSummary, PlanConfirmation, PlanConfirmationError, PlanPreview, PlanProposal,
    PlanProposalError, PlanWorkItemPreview,
};
pub use planner::{
    MAX_PLAN_ASSUMPTIONS, MAX_PLAN_DEPENDENCIES, MAX_PLAN_WORK_ITEMS, MAX_PLANNER_GOAL_BYTES,
    PlanDraft, PlanDraftDependency, PlanDraftError, PlanDraftWorkItem, PlannerProfile,
    PlannerProfileError,
};
pub use planner_process::{ProcessPlanGenerationError, ProcessPlanGenerator};
pub use scheduler::{
    DaemonScheduler, PolicyDeferredWorkItem, RepositoryDeferredWorkItem, ScheduledLaunch,
    SchedulerError, SchedulerResult, SchedulerTick,
};
pub use supervisor::{
    BoardSupervisionInput, BoardSupervisionInputError, MAX_SUPERVISION_ACTIVITY,
    MAX_SUPERVISION_EVIDENCE, SupervisionActivity, SupervisionCandidate, SupervisionDependency,
    SupervisionEvidence, SupervisionWorkItem, SupervisorRecommendation,
    SupervisorRecommendationError, bounded_summary, redacted_summary,
};
pub use supervisor_process::{ProcessBoardSupervisionError, ProcessBoardSupervisor};
pub use ticket_effect::{
    TicketEffectEvidence, TicketEffectInput, TicketEffectInputError, TicketEffectRecommendation,
    TicketEffectRecommendationError, TicketEffectTask, bounded_redacted,
};
pub use ticket_effect_process::{ProcessTicketEffectAdvisor, ProcessTicketEffectError};

#[cfg(test)]
mod tests;

#[cfg(all(test, unix))]
mod planner_process_tests;

#[cfg(all(test, unix))]
mod supervisor_process_tests;

#[cfg(all(test, unix))]
mod ticket_effect_process_tests;
