use std::cell::RefCell;

use anyhow::Context;
use parking_lot::Mutex;
use tilepad_plugin_sdk::Inspector;
use twitch_api::{
    HelixClient,
    helix::chat::{SendChatMessageBody, SendChatMessageRequest, SendChatMessageResponse},
    twitch_oauth2::{AccessToken, UserToken},
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
}

impl State {
    pub fn set_inspector(&self, inspector: Option<Inspector>) {
        *self.inspector.borrow_mut() = inspector;
    }

    pub fn set_logged_out(&self) {
        let state = &mut *self.access_state.lock();
        *state = AccessState::NotAuthenticate;
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
}
