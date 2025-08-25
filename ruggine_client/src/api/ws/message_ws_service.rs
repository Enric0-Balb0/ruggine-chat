use crate::types::message_ws::*;
use leptos::logging::log;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket, Event, ErrorEvent, CloseEvent};
use std::rc::Rc;
use std::cell::RefCell;

/// Stato della connessione WebSocket
#[derive(Debug, Clone, PartialEq)]
pub enum WsStatus {
    Connecting,
    Open,
    Closed,
    Error(String),
}

/// Service base per gestire la connessione WebSocket ai messaggi di gruppo
pub struct MessageWsService {
    ws: Option<WebSocket>,
    status: Rc<RefCell<WsStatus>>,
    on_message: Option<Box<dyn Fn(WebSocketMessage) + 'static>>,
}

impl MessageWsService {
    pub fn new() -> Self {
        Self {
            ws: None,
            status: Rc::new(RefCell::new(WsStatus::Closed)),
            on_message: None,
        }
    }

    /// Imposta la callback per la ricezione dei messaggi
    pub fn set_on_message<F>(&mut self, callback: F)
    where
        F: Fn(WebSocketMessage) + 'static,
    {
        self.on_message = Some(Box::new(callback));
    }

    /// Connette al WS (url completo, es: ws://...)
    pub fn connect(&mut self, url: &str) {
        let ws = WebSocket::new(url).expect("WebSocket creation failed");
        *self.status.borrow_mut() = WsStatus::Connecting;
        // Setup eventi base
        let status = self.status.clone();
        let onopen = Closure::wrap(Box::new(move |_e: Event| {
            *status.borrow_mut() = WsStatus::Open;
            log!("WebSocket aperto");
        }) as Box<dyn FnMut(_)>);
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();
        let status = self.status.clone();
        let onerror = Closure::wrap(Box::new(move |e: ErrorEvent| {
            *status.borrow_mut() = WsStatus::Error(e.message());
            log!("WebSocket errore: {:?}", e.message());
        }) as Box<dyn FnMut(_)>);
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();
        let status = self.status.clone();
        let onclose = Closure::wrap(Box::new(move |_e: CloseEvent| {
            *status.borrow_mut() = WsStatus::Closed;
            log!("WebSocket chiuso");
        }) as Box<dyn FnMut(_)>);
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();

        // Gestione ricezione messaggi
        let on_message_cb = self.on_message.as_ref().map(|cb| cb as *const _);
        let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let txt: String = txt.into();
                match serde_json::from_str::<WebSocketMessage>(&txt) {
                    Ok(msg) => {
                        if let Some(cb_ptr) = on_message_cb {
                            // Safety: cb_ptr è valido finché self vive
                            let cb: &Box<dyn Fn(WebSocketMessage)> = unsafe { &*cb_ptr };
                            cb(msg);
                        }
                    }
                    Err(e) => {
                        log!("Errore parsing messaggio WS: {:?}", e);
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        self.ws = Some(ws);
    }

    /// Invia un messaggio serializzato
    pub fn send(&self, msg: &WebSocketMessage) {
        if let Some(ws) = &self.ws {
            let data = serde_json::to_string(msg).expect("serialize ws msg");
            let _ = ws.send_with_str(&data);
        }
    }

    /// Chiude la connessione
    pub fn disconnect(&mut self) {
        if let Some(ws) = &self.ws {
            let _ = ws.close();
        }
        self.ws = None;
        *self.status.borrow_mut() = WsStatus::Closed;
    }

    pub fn status(&self) -> WsStatus {
        self.status.borrow().clone()
    }
}
