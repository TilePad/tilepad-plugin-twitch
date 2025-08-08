use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::{Duration, Instant},
};

use anyhow::Context;
use parking_lot::Mutex;
use serde::Serialize;
use tilepad_plugin_sdk::{Display, Inspector, tracing};
use tokio::time::sleep;
use twitch_api::{
    HelixClient,
    helix::{
        EmptyBody, Request, RequestPost, Scope,
        chat::{
            ChatSettings, GetChatSettingsRequest, SendChatMessageBody, SendChatMessageRequest,
            SendChatMessageResponse, UpdateChatSettingsBody, UpdateChatSettingsRequest,
        },
        clips::{CreateClipRequest, CreatedClip},
        moderation::{DeleteChatMessagesRequest, DeleteChatMessagesResponse},
        streams::{
            CreateStreamMarkerBody, CreateStreamMarkerRequest, CreatedStreamMarker,
            GetStreamsRequest,
        },
    },
    twitch_oauth2::{AccessToken, UserToken, Validator, validator},
};

use crate::messages::InspectorMessageOut;

#[derive(Default)]
#[allow(clippy::large_enum_variant)]
pub enum AccessState {
    NotAuthenticate,
    #[default]
    Loading,
    Authenticated {
        user_token: UserToken,
    },
}

#[derive(Default)]
pub struct State {
    helix_client: HelixClient<'static, reqwest::Client>,
    access_state: Mutex<AccessState>,
    inspector: RefCell<Option<Inspector>>,

    view_displays: RefCell<Vec<ViewCountDisplay>>,
    viewers: Cell<usize>,
}

impl State {
    pub fn set_inspector(&self, inspector: Option<Inspector>) {
        *self.inspector.borrow_mut() = inspector;
    }

    pub fn set_logged_out(&self) {
        let state = &mut *self.access_state.lock();
        *state = AccessState::NotAuthenticate;
        self.update_inspector();
    }

    pub fn update_inspector(&self) {
        if let Some(inspector) = self.inspector.borrow().as_ref() {
            let state = &*self.access_state.lock();
            match state {
                AccessState::NotAuthenticate => {
                    _ = inspector.send(InspectorMessageOut::State {
                        state: "NOT_AUTHENTICATED".to_string(),
                    });
                }
                AccessState::Loading => {
                    _ = inspector.send(InspectorMessageOut::State {
                        state: "LOADING".to_string(),
                    });
                }
                AccessState::Authenticated { .. } => {
                    _ = inspector.send(InspectorMessageOut::State {
                        state: "AUTHENTICATED".to_string(),
                    });
                }
            }
        }
    }

    pub async fn create_user_token(&self, access_token: AccessToken) -> anyhow::Result<UserToken> {
        UserToken::from_existing(&self.helix_client, access_token, None, None)
            .await
            .context("failed to create user token")
    }

    pub async fn attempt_auth(&self, access_token: AccessToken) -> anyhow::Result<()> {
        {
            let lock = &mut *self.access_state.lock();
            *lock = AccessState::Loading;
        }

        self.update_inspector();

        // Create user token (Validates it with the twitch backend)
        let user_token = self.create_user_token(access_token).await?;

        {
            let lock = &mut *self.access_state.lock();
            *lock = AccessState::Authenticated { user_token };
        }

        self.update_inspector();

        Ok(())
    }

    pub fn get_user_token(&self) -> Option<UserToken> {
        let lock = &*self.access_state.lock();
        match lock {
            AccessState::Authenticated { user_token } => Some(user_token.clone()),
            _ => None,
        }
    }

    pub async fn send_chat_message(
        &self,
        message: &str,
    ) -> anyhow::Result<SendChatMessageResponse> {
        // Obtain twitch access token
        let token = self.get_user_token().context("not authenticated")?;

        // Get broadcaster user ID
        let user_id = token.user_id.clone();

        // Create chat message request
        let request = SendChatMessageRequest::new();
        let body = SendChatMessageBody::new(user_id.clone(), user_id, message);

        // Send request and get response
        let response: SendChatMessageResponse = self
            .helix_client
            .req_post(request, body, &token)
            .await?
            .data;

        Ok(response)
    }

    /// Sends a message to Twitch chat, if the message is over the 500 character limit
    /// the message will be chunked into multiple parts and sent separately
    pub async fn send_chat_message_chunked(&self, message: &str) -> anyhow::Result<()> {
        if message.len() < 500 {
            self.send_chat_message(message).await?;
        } else {
            let mut chars = message.chars();
            loop {
                let message = chars.by_ref().take(500).collect::<String>();
                if message.is_empty() {
                    break;
                }

                self.send_chat_message(&message).await?;
            }
        }

        Ok(())
    }

    pub async fn clear_chat(&self) -> anyhow::Result<DeleteChatMessagesResponse> {
        let token = self.get_user_token().context("not authenticated")?;
        let user_id = token.user_id.clone();
        let request = DeleteChatMessagesRequest::new(user_id.clone(), user_id);
        let response: DeleteChatMessagesResponse =
            self.helix_client.req_delete(request, &token).await?.data;

        Ok(response)
    }

    pub async fn create_clip(&self) -> anyhow::Result<Vec<CreatedClip>> {
        let token = self.get_user_token().context("not authenticated")?;
        let user_id = token.user_id.clone();
        let request = CreateClipRequestFixed(CreateClipRequest::broadcaster_id(user_id));
        let response: Vec<CreatedClip> = self
            .helix_client
            .req_post(request, EmptyBody, &token)
            .await?
            .data;

        Ok(response)
    }

    pub async fn create_marker(&self, description: String) -> anyhow::Result<CreatedStreamMarker> {
        let token = self.get_user_token().context("not authenticated")?;
        let user_id = token.user_id.clone();
        let request = CreateStreamMarkerRequest::new();
        let body = CreateStreamMarkerBody::new(user_id, description);
        let response: CreatedStreamMarker = self
            .helix_client
            .req_post(request, body, &token)
            .await?
            .data;

        Ok(response)
    }

    pub async fn get_chat_settings(&self) -> anyhow::Result<ChatSettings> {
        let token = self.get_user_token().context("not authenticated")?;
        let user_id = token.user_id.clone();
        let request = GetChatSettingsRequest::broadcaster_id(user_id.clone());
        let response: ChatSettings = self.helix_client.req_get(request, &token).await?.data;
        Ok(response)
    }

    pub async fn toggle_slow_mode(&self) -> anyhow::Result<()> {
        let settings = self.get_chat_settings().await?;
        let token = self.get_user_token().context("not authenticated")?;
        let user_id = token.user_id.clone();
        let request = UpdateChatSettingsRequest::new(user_id.clone(), user_id);
        let mut body = UpdateChatSettingsBody::default();
        body.slow_mode = Some(!settings.slow_mode);

        _ = self.helix_client.req_patch(request, body, &token).await?;
        Ok(())
    }

    pub async fn toggle_emote_only(&self) -> anyhow::Result<()> {
        let settings = self.get_chat_settings().await?;
        let token = self.get_user_token().context("not authenticated")?;
        let user_id = token.user_id.clone();
        let request = UpdateChatSettingsRequest::new(user_id.clone(), user_id);
        let mut body = UpdateChatSettingsBody::default();
        body.emote_mode = Some(!settings.emote_mode);

        _ = self.helix_client.req_patch(request, body, &token).await?;
        Ok(())
    }

    pub async fn toggle_follower_only(&self) -> anyhow::Result<()> {
        let settings = self.get_chat_settings().await?;
        let token = self.get_user_token().context("not authenticated")?;
        let user_id = token.user_id.clone();
        let request = UpdateChatSettingsRequest::new(user_id.clone(), user_id);
        let mut body = UpdateChatSettingsBody::default();
        body.follower_mode = Some(!settings.follower_mode);

        _ = self.helix_client.req_patch(request, body, &token).await?;
        Ok(())
    }

    pub async fn toggle_sub_only(&self) -> anyhow::Result<()> {
        let settings = self.get_chat_settings().await?;
        let token = self.get_user_token().context("not authenticated")?;
        let user_id = token.user_id.clone();
        let request = UpdateChatSettingsRequest::new(user_id.clone(), user_id);
        let mut body = UpdateChatSettingsBody::default();
        body.subscriber_mode = Some(!settings.subscriber_mode);

        _ = self.helix_client.req_patch(request, body, &token).await?;
        Ok(())
    }

    pub async fn get_view_count(&self) -> anyhow::Result<Option<usize>> {
        let token = match self.get_user_token() {
            Some(value) => value,
            None => return Ok(None),
        };
        let user_id = token.user_id.clone();
        let request = GetStreamsRequest::user_ids(vec![user_id]).first(1);

        let response = self.helix_client.req_get(request, &token).await?.data;
        let view_count = response.first().map(|stream| stream.viewer_count);
        Ok(view_count)
    }

    // Returning the number of active ones
    pub fn get_active_displays(&self) -> usize {
        let now = Instant::now();
        let displays = &mut *self.view_displays.borrow_mut();
        displays.retain(|display| now.duration_since(display.last_alive) < Duration::from_secs(5));

        displays.len()
    }

    pub fn current_view_count(&self) -> usize {
        self.viewers.get()
    }

    pub fn push_active_display(&self, display: &Display) {
        let displays = &mut *self.view_displays.borrow_mut();
        let now = Instant::now();

        if let Some(existing) = displays
            .iter_mut()
            .find(|other| other.display.ctx.eq(&display.ctx))
        {
            existing.last_alive = now;
        } else {
            displays.push(ViewCountDisplay {
                display: display.clone(),
                last_alive: now,
            });
        }
    }
}

/// Wrapper to correct the HTTP method type for the create clip endpoint
#[derive(Serialize)]
#[serde(transparent)]
struct CreateClipRequestFixed<'a>(CreateClipRequest<'a>);

impl Request for CreateClipRequestFixed<'_> {
    type Response = Vec<CreatedClip>;

    const PATH: &'static str = "clips";
    const SCOPE: Validator = validator![Scope::ClipsEdit];
}

impl RequestPost for CreateClipRequestFixed<'_> {
    type Body = EmptyBody;
}

pub struct ViewCountDisplay {
    display: Display,
    last_alive: Instant,
}

pub async fn run_view_count_update(state: Rc<State>) {
    loop {
        let active = state.get_active_displays();

        if active > 0 {
            let view_count = match state.get_view_count().await {
                Ok(value) => value,
                Err(error) => {
                    tracing::error!(?error, "failed to get view count");
                    None
                }
            };

            if let Some(view_count) = view_count {
                state.viewers.replace(view_count);
            }
        }

        // Update every 5 seconds
        sleep(Duration::from_secs(5)).await;
    }
}
