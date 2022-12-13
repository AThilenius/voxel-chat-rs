use bevy::prelude::*;

mod ws;

use serde::{Deserialize, Serialize};

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ws::NetWsPlugin::default())
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
