use plugin::ExamplePlugin;
use tilepad_plugin_sdk::{setup_tracing, start_plugin};
use tokio::task::LocalSet;

pub mod action;
pub mod messages;
pub mod plugin;
pub mod state;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Setup tracing
    setup_tracing();

    let local_set = LocalSet::new();
    let plugin = ExamplePlugin::new();

    local_set.run_until(start_plugin(plugin)).await;
}
