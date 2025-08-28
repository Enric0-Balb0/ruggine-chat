// pub fn use_global_group_unread_ws(group_ids: Vec<i32>) {

use leptos::*;
use crate::context::unread_counts_context::use_unread_counts_context;

/// Hook placeholder per la gestione globale dei badge non letti.
/// Al momento è una no-op; sarà implementata per collegare i websocket globali.
pub fn use_global_group_unread_ws(_group_ids: Vec<i32>) {
    // Placeholder: global unread badge hook.
    // Actual unread increments happen in the WebSocket hook `use_group_message_ws`.
}

// No wasm exports: this hook is currently a no-op and does not expose JS bindings.
