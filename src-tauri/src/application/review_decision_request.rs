use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordReviewDecisionRequest {
    pub evidence_id: String,
    pub work_item_id: String,
    pub reviewer: String,
    pub summary: String,
    pub accepted: bool,
    pub recorded_at: String,
}
