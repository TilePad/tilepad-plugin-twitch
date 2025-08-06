use serde::{Deserialize, Serialize};

/// Messages from the inspector
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InspectorMessageIn {
    GetState,
    OpenAuthUrl,
}

/// Messages to the inspector
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InspectorMessageOut {
    State { state: String },
}
