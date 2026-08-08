use serde::Serialize;

use crate::orchestration::{PlanConfirmation, PlanPreview, PlanProposal};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardPlan {
    pub preview: PlanPreview,
    pub confirmation: Option<PlanConfirmation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredPlan {
    pub proposal: PlanProposal,
    pub confirmation: Option<PlanConfirmation>,
}
