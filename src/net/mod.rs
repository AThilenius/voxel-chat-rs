use bevy::prelude::*;

// #[cfg(target_arch = "wasm32")]
mod net_wasm;

// #[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app
            // #[cfg(target_arch = "wasm32")]
            .add_plugin(net_wasm::NetWasmPlugin::default())
            .add_system(sample_consumer);
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NetMsg {
    test: String,
}

/// Common event type for inbound messages.
#[derive(Debug)]
pub struct InboundNetMsg(NetMsg);

fn sample_consumer(mut reader: EventReader<InboundNetMsg>) {
    for msg in reader.iter() {
        info!("Consumed ECS WS message! Woo hoo: {:?}", msg.0);
    }
}
