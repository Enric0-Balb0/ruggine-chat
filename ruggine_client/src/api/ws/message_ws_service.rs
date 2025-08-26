use crate::types::message_ws::*;
use leptos::logging::log;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket, Event, ErrorEvent, CloseEvent};
use leptos::{RwSignal, SignalSet, SignalGet};

use crate::types::message_ws::WsStatus;

// WebSocket service for managing group messages
pub struct MessageWsService {
    ws: Option<WebSocket>,
    status: RwSignal<WsStatus>,
    on_message: Option<Box<dyn Fn(WebSocketMessage) + 'static>>,
}

impl MessageWsService {
    pub fn new(status: RwSignal<WsStatus>) -> Self {
        Self {
            ws: None,
            status,
            on_message: None,
        }
    }

    // Set the callback for receiving messages
    pub fn set_on_message<F>(&mut self, callback: F)
    where
        F: Fn(WebSocketMessage) + 'static,
    {
        self.on_message = Some(Box::new(callback));
    }

    // Connect to the WebSocket (full url, e.g. ws://...)
    pub fn connect(&mut self, url: &str) {
        let ws = WebSocket::new(url).expect("WebSocket creation failed");
        self.status.set(WsStatus::Connecting);
   
    // Setup basic WebSocket events
        let status = self.status;
        let url_clone = url.to_string();
        let onopen = Closure::wrap(Box::new(move |_e: Event| {
            status.set(WsStatus::Open);
        }) as Box<dyn FnMut(_)>);
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();
        let status = self.status;
        let url_clone = url.to_string();
        let onerror = Closure::wrap(Box::new(move |e: ErrorEvent| {
            status.set(WsStatus::Error(e.message()));
        }) as Box<dyn FnMut(_)>);
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();
        let status = self.status;
        let url_clone = url.to_string();
        let onclose = Closure::wrap(Box::new(move |e: CloseEvent| {
            status.set(WsStatus::Closed);
        }) as Box<dyn FnMut(_)>);
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();

    // Handle incoming messages
        let on_message_cb = self.on_message.as_ref().map(|cb| cb as *const _);
        let url_clone = url.to_string();
        let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let txt: String = txt.into();
                match serde_json::from_str::<WebSocketMessage>(&txt) {
                    Ok(msg) => {
                        if let Some(cb_ptr) = on_message_cb {
                            // SAFETY: cb_ptr is valid as long as self lives
                            let cb: &Box<dyn Fn(WebSocketMessage)> = unsafe { &*cb_ptr };
                            cb(msg);
                        }
                    }
                    Err(e) => {
                        log!("[WS] Errore parsing messaggio WS su {}: {:?}", url_clone, e);
                    }
                }
            } else {
                log!("[WS] Messaggio ricevuto non stringa su {}", url_clone);
            }
        }) as Box<dyn FnMut(_)>);
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        self.ws = Some(ws);
    }

    // Send a serialized message only if the connection is Open
    pub fn send(&self, msg: &WebSocketMessage) {
        let current_status = self.status();
        if let Some(ws) = &self.ws {
            if current_status == WsStatus::Open {
                let data = serde_json::to_string(msg).expect("serialize ws msg");
                match ws.send_with_str(&data) {
                    Ok(_) => (),
                    Err(_e) => (),
                }
            }
        }
    }

    // Close the connection
    pub fn disconnect(&mut self) {
        if let Some(ws) = &self.ws {
            let _ = ws.close();
        }
    self.ws = None;
    self.status.set(WsStatus::Closed);
    }

    pub fn status(&self) -> WsStatus {
        let s = self.status.get();
        s
    }
}
