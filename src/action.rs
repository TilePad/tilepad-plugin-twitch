use serde::Deserialize;

pub enum Action {
    SendMessage(SendMessageProperties),
}

impl Action {
    pub fn from_action(
        action_id: &str,
        properties: serde_json::Value,
    ) -> Option<Result<Action, serde_json::Error>> {
        Some(match action_id {
            "send_message" => serde_json::from_value(properties).map(Action::SendMessage),
            _ => return None,
        })
    }
}

#[derive(Deserialize)]
pub struct SendMessageProperties {
    pub message: Option<String>,
}
