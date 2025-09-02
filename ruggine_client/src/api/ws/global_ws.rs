use leptos::*;
use leptos::ReadSignal;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::api::MessageWsService;
use crate::types::message_ws::WsStatus;

// Single-threaded global map — avoid Sync/Send requirements for WASM by using
// a leaked Box stored behind a raw pointer. This is acceptable for a short-
// lived dev client use. If server-side multi-threading is needed, replace with
// a proper thread-safe container.
fn global_map() -> &'static RefCell<HashMap<String, Rc<RefCell<MessageWsService>>>> {
    static mut MAP_PTR: *const RefCell<HashMap<String, Rc<RefCell<MessageWsService>>>> = 0 as *const _;
    unsafe {
        if MAP_PTR.is_null() {
            let b = Box::new(RefCell::new(HashMap::new()));
            MAP_PTR = Box::into_raw(b);
        }
        &*MAP_PTR
    }
}

/// Get or create a shared MessageWsService for a token.
/// Returns (service, read_only_status_signal).
pub fn get_or_create(token: &str) -> (Rc<RefCell<MessageWsService>>, ReadSignal<WsStatus>) {
    let map_cell = global_map();
    let mut map = map_cell.borrow_mut();
    if let Some(svc) = map.get(token) {
        let svc = svc.clone();
        let status = svc.borrow().status_signal().expect("status present");
    return (svc, status);
    }

    // Must be called inside a leptos runtime
    let status = create_rw_signal(WsStatus::Connecting);
    let svc = Rc::new(RefCell::new(MessageWsService::new(status)));
    let status_read = svc.borrow().status_signal().expect("status present");
    map.insert(token.to_string(), svc.clone());
    (svc, status_read)
}

/// Disconnect and remove the shared service for a specific token if present.
pub fn disconnect_for_token(token: &str) {
    let map_cell = global_map();
    let mut map = map_cell.borrow_mut();
    if let Some(svc) = map.remove(token) {
        svc.borrow_mut().disconnect();
    }
}

/// Disconnect all managed services and clear the global registry.
pub fn disconnect_all() {
    let map_cell = global_map();
    let mut map = map_cell.borrow_mut();
    for (_k, svc) in map.drain() {
        svc.borrow_mut().disconnect();
    }
}
