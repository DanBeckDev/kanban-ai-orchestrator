use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordCleanCodeReviewRequest {
    pub evidence_id: String,
    pub work_item_id: String,
    pub review_execution_id: String,
    pub actionable_finding_count: u32,
    pub summary: String,
    pub recorded_at: String,
}
