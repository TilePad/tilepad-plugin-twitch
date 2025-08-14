use crate::{
    action::Action,
    messages::{DisplayMessageIn, DisplayMessageOut, InspectorMessageIn},
    state::{State, run_view_count_update},
};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use tilepad_plugin_sdk::{
    Inspector, Plugin, PluginSessionHandle, TileInteractionContext,
    tracing::{self},
};
use tokio::task::spawn_local;
use twitch_api::{
    helix::Scope,
    twitch_oauth2::{AccessToken, ImplicitUserTokenBuilder, types::ClientIdRef, url::Url},
};

/// If you are forking this app program for your own use, please create your own
/// twitch developer application client ID at https://dev.twitch.tv/console/apps
pub const TWITCH_CLIENT_ID: &ClientIdRef =
    ClientIdRef::from_static("yr9puvx670aq6m8beggiakivxob6tx");

/// Scopes required from twitch by the app
pub const TWITCH_REQUIRED_SCOPES: &[Scope] = &[
    // Send chat messages
    Scope::UserWriteChat,
    // Allow deleting messages
    Scope::ModeratorManageChatMessages,
];

/// Properties for the plugin itself
#[derive(Debug, Deserialize, Serialize)]
pub struct Properties {
    access: Option<StoredAccess>,
}

#[derive(Debug, Deserialize, Serialize)]
struct StoredAccess {
    access_token: AccessToken,
    scopes: Vec<Scope>,
}

#[derive(Default)]
pub struct ExamplePlugin {
    state: Rc<State>,
}

impl ExamplePlugin {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Plugin for ExamplePlugin {
    fn on_registered(&mut self, _session: &PluginSessionHandle) {
        spawn_local(run_view_count_update(self.state.clone()));
    }

    fn on_properties(&mut self, _session: &PluginSessionHandle, properties: serde_json::Value) {
        let state = self.state.clone();
        let properties: Properties = match serde_json::from_value(properties) {
            Ok(value) => value,
            Err(cause) => {
                tracing::error!(?cause, "failed to parse properties");
                return;
            }
        };

        state.set_logged_out();

        // Try and authenticate
        spawn_local(async move {
            if let Some(stored) = properties.access {
                if let Err(error) = state.attempt_auth(stored.access_token).await {
                    // TODO: If token is bad delete and force re-login
                    tracing::error!(?error, "auth attempt failed");
                }
            }
        });
    }

    fn on_inspector_open(&mut self, _session: &PluginSessionHandle, inspector: Inspector) {
        self.state.set_inspector(Some(inspector));
    }

    fn on_inspector_close(&mut self, _session: &PluginSessionHandle, _inspector: Inspector) {
        self.state.set_inspector(None);
    }

    fn on_inspector_message(
        &mut self,
        session: &PluginSessionHandle,
        _inspector: Inspector,
        message: serde_json::Value,
    ) {
        let message: InspectorMessageIn = match serde_json::from_value(message) {
            Ok(value) => value,
            Err(_) => return,
        };

        match message {
            InspectorMessageIn::GetState => {
                self.state.update_inspector();
            }
            InspectorMessageIn::OpenAuthUrl => {
                let redirect_url =
                    Url::parse("https://tilepad.pages.dev/deep-link/com.jacobtread.tilepad.twitch")
                        .expect("redirect url is hardcoded and must be valid");

                let (url, _csrf) =
                    ImplicitUserTokenBuilder::new(TWITCH_CLIENT_ID.into(), redirect_url)
                        .set_scopes(TWITCH_REQUIRED_SCOPES.to_vec())
                        .generate_url();

                _ = session.open_url(url.to_string());
            }
            InspectorMessageIn::Logout => {
                self.state.set_logged_out();
                _ = session.set_properties(Properties { access: None });
            }
        }
    }

    fn on_display_message(
        &mut self,
        _session: &PluginSessionHandle,
        display: tilepad_plugin_sdk::Display,
        message: serde_json::Value,
    ) {
        let message: DisplayMessageIn = match serde_json::from_value(message) {
            Ok(value) => value,
            Err(_) => return,
        };

        match message {
            DisplayMessageIn::GetViewCount => {
                self.state.push_active_display(&display);

                _ = display.send(DisplayMessageOut::ViewCount {
                    count: self.state.current_view_count(),
                });
            }
        }
    }

    fn on_tile_clicked(
        &mut self,
        _session: &PluginSessionHandle,
        ctx: TileInteractionContext,
        properties: serde_json::Value,
    ) {
        let action_id = ctx.action_id.as_str();
        let action = match Action::from_action(action_id, properties) {
            Some(Ok(value)) => value,
            Some(Err(cause)) => {
                tracing::error!(?cause, ?action_id, "failed to deserialize action");
                return;
            }
            None => {
                tracing::debug!(?action_id, "unknown tile action requested");
                return;
            }
        };

        let state = self.state.clone();

        match action {
            Action::SendMessage(properties) => {
                spawn_local(async move {
                    let message = match properties.message {
                        Some(value) => value,
                        None => return,
                    };

                    if let Err(error) = state.send_chat_message(&message).await {
                        tracing::error!(?error, "failed to send chat message");
                    }
                });
            }
            Action::ClearChat => {
                spawn_local(async move {
                    if let Err(error) = state.clear_chat().await {
                        tracing::error!(?error, "failed to clear chat");
                    }
                });
            }
            Action::EmoteOnly => {
                spawn_local(async move {
                    if let Err(error) = state.toggle_emote_only().await {
                        tracing::error!(?error, "failed to toggle emote only chat");
                    }
                });
            }
            Action::FollowerOnly => {
                spawn_local(async move {
                    if let Err(error) = state.toggle_follower_only().await {
                        tracing::error!(?error, "failed to toggle follower only chat");
                    }
                });
            }
            Action::SubOnly => {
                spawn_local(async move {
                    if let Err(error) = state.toggle_sub_only().await {
                        tracing::error!(?error, "failed to toggle sub only chat");
                    }
                });
            }
            Action::SlowMode => {
                spawn_local(async move {
                    if let Err(error) = state.toggle_slow_mode().await {
                        tracing::error!(?error, "failed to toggle slow mode");
                    }
                });
            }
            Action::AdBreak(properties) => {
                spawn_local(async move {
                    if let Err(error) = state
                        .start_comercial(
                            properties
                                .length
                                .unwrap_or(twitch_api::types::CommercialLength::Length30),
                        )
                        .await
                    {
                        tracing::error!(?error, "failed to create marker");
                    }
                });
            }
            Action::Marker(properties) => {
                spawn_local(async move {
                    if let Err(error) = state
                        .create_marker(properties.description.unwrap_or_default())
                        .await
                    {
                        tracing::error!(?error, "failed to create marker");
                    }
                });
            }
            Action::CreateClip => {
                spawn_local(async move {
                    if let Err(error) = state.create_clip().await {
                        tracing::error!(?error, "failed to create clip");
                    }
                });
            }
            Action::OpenClip => {}
            Action::ViewerCount => {
                // No associated action (Maybe refresh manually when tapped?)
            }
        }
    }

    fn on_deep_link(
        &mut self,
        session: &PluginSessionHandle,
        ctx: tilepad_plugin_sdk::DeepLinkContext,
    ) {
        // Fragment is required
        let fragment = match ctx.fragment {
            Some(value) => value,
            None => return,
        };

        let fragment: DeepLinkFragment = match serde_urlencoded::from_str(&fragment) {
            Ok(value) => value,
            Err(_) => return,
        };

        let access_token = fragment.access_token;
        let scopes: Vec<Scope> = fragment
            .scope
            .split(':')
            .map(|scope| Scope::parse(scope.to_string()))
            .collect();

        // Try authenticates
        let session = session.clone();
        let state = self.state.clone();
        spawn_local(async move {
            if let Err(error) = state.attempt_auth(access_token.clone()).await {
                tracing::error!(?error, "failed to authenticate");
                return;
            }

            // Store authentication credentials
            _ = session.set_properties(Properties {
                access: Some(StoredAccess {
                    access_token,
                    scopes,
                }),
            });
        });
    }
}

#[derive(Debug, Deserialize)]
struct DeepLinkFragment {
    access_token: AccessToken,
    scope: String,
}
