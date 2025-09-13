use leptos::*;
use leptos::ReadSignal;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::api::MessageWsService;
use crate::types::message_ws::WsStatus;

// Global state for tracking join confirmations across all instances
struct GlobalJoinState {
    last_join_request_id: Option<String>,
    join_confirmed: bool,
}

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

fn global_join_state() -> &'static RefCell<HashMap<String, GlobalJoinState>> {
    static mut JOIN_STATE_PTR: *const RefCell<HashMap<String, GlobalJoinState>> = 0 as *const _;
    unsafe {
        if JOIN_STATE_PTR.is_null() {
            let b = Box::new(RefCell::new(HashMap::new()));
            JOIN_STATE_PTR = Box::into_raw(b);
        }
        &*JOIN_STATE_PTR
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

/// Reset join tracking state for a token (clear stale state on login)
pub fn reset_join_tracking(token: &str) {
    let join_state_cell = global_join_state();
    let mut join_map = join_state_cell.borrow_mut();
    join_map.insert(token.to_string(), GlobalJoinState {
        last_join_request_id: None,
        join_confirmed: false,
    });
    leptos::logging::log!("[GLOBAL WS] Reset join tracking for token");
}

/// Set the last join request ID for a token
pub fn set_join_request_id(token: &str, request_id: String) {
    let join_state_cell = global_join_state();
    let mut join_map = join_state_cell.borrow_mut();
    let state = join_map.entry(token.to_string()).or_insert(GlobalJoinState {
        last_join_request_id: None,
        join_confirmed: false,
    });
    state.last_join_request_id = Some(request_id.clone());
    state.join_confirmed = false;
    leptos::logging::log!("[GLOBAL WS] Set join request ID: {}", request_id);
}

/// Mark join as confirmed for a token if the request ID matches
pub fn confirm_join(token: &str, request_id: &str) -> bool {
    let join_state_cell = global_join_state();
    let mut join_map = join_state_cell.borrow_mut();
    if let Some(state) = join_map.get_mut(token) {
        if let Some(ref last_req) = state.last_join_request_id {
            if last_req == request_id {
                state.join_confirmed = true;
                leptos::logging::log!("[GLOBAL WS] Join confirmed for token, request_id: {}", request_id);
                return true;
            }
        }
    }
    false
}

/// Check if join is confirmed for a token
pub fn is_join_confirmed(token: &str) -> bool {
    let join_state_cell = global_join_state();
    let join_map = join_state_cell.borrow();
    join_map.get(token).map_or(false, |state| state.join_confirmed)
}

/// Get the last join request ID for a token
pub fn get_last_join_request_id(token: &str) -> Option<String> {
    let join_state_cell = global_join_state();
    let join_map = join_state_cell.borrow();
    join_map.get(token).and_then(|state| state.last_join_request_id.clone())
}

/// Disconnect and remove the shared service for a specific token if present.
pub fn disconnect_for_token(token: &str) {
    let map_cell = global_map();
    let mut map = map_cell.borrow_mut();
    if let Some(svc) = map.remove(token) {
        svc.borrow_mut().disconnect();
    }
    
    // Also clear join tracking state
    let join_state_cell = global_join_state();
    let mut join_map = join_state_cell.borrow_mut();
    join_map.remove(token);
}

/// Disconnect all managed services and clear the global registry.
pub fn disconnect_all() {
    let map_cell = global_map();
    let mut map = map_cell.borrow_mut();
    for (_k, svc) in map.drain() {
        svc.borrow_mut().disconnect();
    }
    
    // Clear all join tracking state
    let join_state_cell = global_join_state();
    let mut join_map = join_state_cell.borrow_mut();
    join_map.clear();
}
