use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use bevy::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

use crate::net::NetMsg;

use super::InboundNetMsg;

#[derive(Default)]
pub struct NetWasmPlugin;

#[derive(Default, Resource)]
pub struct WasmWebsocket {
    // ws: WebSocket,
    receiver: Option<Arc<Mutex<mpsc::Receiver<NetMsg>>>>,
}

impl Plugin for NetWasmPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WasmWebsocket>()
            .add_event::<InboundNetMsg>()
            .add_startup_system(setup)
            .add_system(dispatch_mpsc_events);
    }
}

fn setup(
    mut wasm_websocket: ResMut<WasmWebsocket>,
    // mut writer: EventWriter<IncomingMsg>,
) {
    let (sender, receiver) = mpsc::channel();
    wasm_websocket.receiver = Some(Arc::new(Mutex::new(receiver)));

    let ws = WebSocket::new("wss://echo.websocket.events").expect("websocket should connect");
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        // Handle difference Text/Binary,...
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            let array = js_sys::Uint8Array::new(&abuf);
            let msg: NetMsg = rmp_serde::from_slice(&array.to_vec())
                .expect("websocket message to be MsgPack encoded");
            sender.send(msg).expect("channel to send");
        } else {
            warn!("message event, received Unknown: {:?}", e.data());
        }
    });

    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

    // TODO: don't actually do this.
    onmessage_callback.forget();

    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        error!("error event: {:?}", e);
    });

    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));

    // TODO: Ditto.
    onerror_callback.forget();

    let cloned_ws = ws.clone();
    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        // Test
        let bytes = rmp_serde::to_vec(&NetMsg {
            test: "Hello, world!".to_string(),
        })
        .unwrap();
        cloned_ws.send_with_u8_array(&bytes).unwrap()
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();
}

fn dispatch_mpsc_events(
    wasm_websocket: Res<WasmWebsocket>,
    mut writer: EventWriter<InboundNetMsg>,
) {
    if let Some(receiver) = &wasm_websocket.receiver {
        let receiver = receiver.lock().expect("lock mpsc websocket receiver mutex");
        writer.send_batch(receiver.try_iter().map(|msg| InboundNetMsg(msg)));
    }
}
