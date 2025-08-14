use serde::Deserialize;
use twitch_api::types::CommercialLength;

pub enum Action {
    SendMessage(SendMessageProperties),
    ClearChat,
    EmoteOnly,
    FollowerOnly,
    SubOnly,
    SlowMode,
    AdBreak(AdBreakProperties),
    Marker(MarkerProperties),
    CreateClip,
    OpenClip,
    ViewerCount,
}

impl Action {
    pub fn from_action(
        action_id: &str,
        properties: serde_json::Value,
    ) -> Option<Result<Action, serde_json::Error>> {
        Some(match action_id {
            "send_message" => serde_json::from_value(properties).map(Action::SendMessage),
            "clear_chat" => Ok(Action::ClearChat),
            "emote_only" => Ok(Action::EmoteOnly),
            "follower_only" => Ok(Action::FollowerOnly),
            "sub_only" => Ok(Action::SubOnly),
            "slow_mode" => Ok(Action::SlowMode),
            "ad_break" => serde_json::from_value(properties).map(Action::AdBreak),
            "marker" => serde_json::from_value(properties).map(Action::Marker),
            "create_clip" => Ok(Action::CreateClip),
            "open_clip" => Ok(Action::OpenClip),
            "viewer_count" => Ok(Action::ViewerCount),
            _ => return None,
        })
    }
}

#[derive(Deserialize)]
pub struct SendMessageProperties {
    pub message: Option<String>,
}

#[derive(Deserialize)]
pub struct MarkerProperties {
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct AdBreakProperties {
    // 1-180s duration of the ad
    pub length: Option<CommercialLength>,
}
