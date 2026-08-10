use serde::Deserialize;

use crate::domain::{TicketEffectAction, TicketEffectResolution};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketEffectPromptRequest {
    pub request_id: String,
    pub work_item_id: String,
    pub action: TicketEffectAction,
    pub prompt: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveTicketEffectRequest {
    pub effect_id: String,
    pub resolution: TicketEffectResolution,
}
