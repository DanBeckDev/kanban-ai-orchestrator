use serde::Deserialize;

/// The deliberate input required to start one worker for one ready task.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartExecutionRequest {
    pub execution_id: String,
    pub work_item_id: String,
    pub agent_profile_name: String,
    pub task_brief: String,
}
