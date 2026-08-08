use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordReviewCheckRequest {
    pub evidence_id: String,
    pub work_item_id: String,
    pub summary: String,
    pub passed: bool,
    pub recorded_at: String,
}
