use crate::types::message_ws::*;
use leptos::logging::log;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket, Event, ErrorEvent, CloseEvent};
use leptos::{RwSignal, SignalSet, SignalGetUntracked, ReadSignal};
use std::rc::Rc;
use std::cell::RefCell;

use crate::types::message_ws::WsStatus;

// WebSocket service for managing group messages
pub struct MessageWsService {
    ws: Option<WebSocket>,
    // store the signal inside an Rc<RefCell<Option<...>>> so closures can check
    // whether the signal still exists before attempting to set it. This avoids
    // updating a signal after its leptos scope has been disposed.
    status: Rc<RefCell<Option<RwSignal<WsStatus>>>>,
    // use an Rc<RefCell<..>> so the JS closure can safely access the list of callbacks
    // This allows multiple consumers (multiple UseGroupMessageWs instances) to
    // register handlers without overwriting each other.
    on_message: Rc<RefCell<Vec<Option<Box<dyn Fn(WebSocketMessage) + 'static>>>>>,
    // Track a single global handler responsible for updating global contexts
    // such as unread counters or invitations to prevent duplicate updates when
    // multiple components attach to the same underlying socket.
    unread_incrementer_id: Rc<RefCell<Option<usize>>>,
}

impl MessageWsService {
    pub fn new(status: RwSignal<WsStatus>) -> Self {
        Self {
            ws: None,
            status: Rc::new(RefCell::new(Some(status))),
            on_message: Rc::new(RefCell::new(Vec::new())),
            unread_incrementer_id: Rc::new(RefCell::new(None)),
        }
    }

    /// Return the internal RwSignal if still present (for read-only access)
    pub fn status_signal(&self) -> Option<ReadSignal<WsStatus>> {
        self.status.borrow().as_ref().map(|s| s.read_only())
    }

    // Register a callback for receiving messages. Multiple callbacks are allowed
    // and will all be invoked for each incoming message.
    pub fn set_on_message<F>(&mut self, callback: F)
    where
        F: Fn(WebSocketMessage) + 'static,
    {
        self.on_message.borrow_mut().push(Some(Box::new(callback)));
        // return the index of the callback as a handle
        // note: caller can ignore the return value if they don't need removal
        // but we keep compatibility by not changing the signature here; we'll
        // provide a separate method to add and return id if needed.
    }

    /// Register a callback and return a numeric id that can be used to remove it later.
    pub fn add_on_message<F>(&mut self, callback: F) -> usize
    where
        F: Fn(WebSocketMessage) + 'static,
    {
        let mut v = self.on_message.borrow_mut();
        v.push(Some(Box::new(callback)));
        v.len() - 1
    }

    /// Try to register a single global handler responsible for updating global
    /// contexts (unread counters, invitations, etc.). Only the first caller
    /// will actually register and receive an id; subsequent calls will return
    /// None so they can avoid duplicating global state updates.
    pub fn add_unread_incrementer<F>(&mut self, callback: F) -> Option<usize>
    where
        F: Fn(WebSocketMessage) + 'static,
    {
        let mut id_slot = self.unread_incrementer_id.borrow_mut();
        if id_slot.is_some() {
            return None;
        }
        let mut v = self.on_message.borrow_mut();
        v.push(Some(Box::new(callback)));
        let id = v.len() - 1;
        *id_slot = Some(id);
        Some(id)
    }

    /// Remove a previously-registered callback by id. Safe to call multiple times.
    pub fn remove_on_message(&mut self, id: usize) {
        if let Some(slot) = self.on_message.borrow_mut().get_mut(id) {
            *slot = None;
        }
        // If the removed id corresponded to the global unread/inviter handler
        // clear the marker so a future consumer can register it again.
        if let Some(current) = *self.unread_incrementer_id.borrow() {
            if current == id {
                *self.unread_incrementer_id.borrow_mut() = None;
            }
        }
    }

    // Connect to the WebSocket (full url, e.g. ws://...)
    pub fn connect(&mut self, url: &str) {
    let svc_ptr = format!("{:p}", self as *const _);
    log!("[SOCKET] Connecting to {} (svc={})", url, svc_ptr);
    let ws = WebSocket::new(url).expect("WebSocket creation failed");

        // set connecting if the signal is still available
        if let Some(s) = self.status.borrow().as_ref() {
            s.set(WsStatus::Connecting);
        }

        // Setup basic WebSocket events
        let status_rc = self.status.clone();
        let url_clone = url.to_string();
        let svc_ptr_clone = svc_ptr.clone();
        let onopen = Closure::wrap(Box::new(move |_e: Event| {
            log!("[SOCKET] Connected to {} (svc={})", url_clone, svc_ptr_clone);
            if let Some(s) = status_rc.borrow().as_ref() {
                s.set(WsStatus::Open);
            }
        }) as Box<dyn FnMut(_)>);
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        let status_rc = self.status.clone();
        let url_clone = url.to_string();
        let svc_ptr_clone = svc_ptr.clone();
        let onerror = Closure::wrap(Box::new(move |e: ErrorEvent| {
            log!("[SOCKET] Error on {} (svc={}): {}", url_clone, svc_ptr_clone, e.message());
            if let Some(s) = status_rc.borrow().as_ref() {
                s.set(WsStatus::Error(e.message()));
            }
        }) as Box<dyn FnMut(_)>);
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();
        let status_rc = self.status.clone();
        let url_clone = url.to_string();
        let svc_ptr_clone = svc_ptr.clone();
        let onclose = Closure::wrap(Box::new(move |e: CloseEvent| {
            log!("[SOCKET] Disconnected from {} (svc={}, code: {}, reason: {})", url_clone, svc_ptr_clone, e.code(), e.reason());
            if let Some(s) = status_rc.borrow().as_ref() {
                s.set(WsStatus::Closed);
            }
        }) as Box<dyn FnMut(_)>);
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();

    // Handle incoming messages. Clone the Rc so the JS closure can access the
    // optionally stored callback at runtime. This avoids creating a raw pointer
    // to a box that may be moved/dropped later and prevents calling into
    // leptos signals after the app has cleaned up.
    let on_message_rc = self.on_message.clone();
    let url_clone = url.to_string();
    let svc_ptr_clone = svc_ptr.clone();
        let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let txt: String = txt.into();
                log!("[SOCKET] Message received on {} (svc={}): {}", url_clone, svc_ptr_clone, txt);
                match serde_json::from_str::<WebSocketMessage>(&txt) {
                        Ok(msg) => {
                            // Borrow the list of callbacks at runtime and call each
                            // callback present. We hold the borrow while invoking
                            // callbacks; callbacks should avoid mutating the list to
                            // prevent borrow conflicts.
                            for cb_opt in on_message_rc.borrow().iter() {
                                if let Some(cb) = cb_opt {
                                    cb(msg.clone());
                                }
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
                log!("[SOCKET] Sending message (svc={}): {}", format!("{:p}", self as *const _), data);
                match ws.send_with_str(&data) {
                    Ok(_) => log!("[SOCKET] Message sent successfully"),
                    Err(e) => log!("[SOCKET] Error sending message: {:?}", e),
                }
            } else {
                log!("[SOCKET] Tried to send message but socket is not open");
            }
        } else {
            log!("[SOCKET] Tried to send message but socket is not initialized");
        }
    }

    // Close the connection
    pub fn disconnect(&mut self) {
        if let Some(ws) = &self.ws {
            log!("[SOCKET] Closing connection (svc={})", format!("{:p}", self as *const _));
            let _ = ws.close();
        } else {
            log!("[SOCKET] Tried to disconnect but socket is not initialized");
        }
        self.ws = None;
    // Clear the on_message callback so any incoming frames that arrive during
    // shutdown won't try to call into leptos signals which may have been dropped.
    // Clear all registered callbacks so they won't be called after disconnect
    self.on_message.borrow_mut().clear();
    // Clear the stored status signal reference so closures won't try to update
    // it after the leptos scope has been disposed.
    self.status.borrow_mut().take();
    // Clear the global unread incrementer marker as well
    self.unread_incrementer_id.borrow_mut().take();
    }

    pub fn status(&self) -> WsStatus {
        // Avoid reactive tracking here; callers expect a plain snapshot. If the
        // stored status signal was already cleared (e.g. after disconnect/cleanup)
        // return Closed.
        if let Some(s) = self.status.borrow().as_ref() {
            s.get_untracked()
        } else {
            WsStatus::Closed
        }
    }
}
