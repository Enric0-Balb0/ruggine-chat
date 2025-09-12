use leptos::*;
use crate::hooks::use_group_message_ws::UseGroupMessageWs;
use crate::types::message_ws::{WebSocketMessage, ServerEvent, GroupEvent};
use crate::api::services::membership::GroupMembershipService;

/// Handler per gestire gli eventi WebSocket di cambio membership (accettazione/rifiuto inviti)
/// Segue il pattern consolidato di chat_view.rs
pub fn create_membership_change_handler(
    ws_hook: UseGroupMessageWs,
    membership_service: GroupMembershipService,
    current_group_id: ReadSignal<Option<i32>>,
    // Callback per aggiornare il contatore quando cambia il numero di membri
    on_membership_change: impl Fn(i32) + Clone + 'static,
) -> impl Fn() + Clone + 'static {
    let on_membership_change = store_value(on_membership_change);
    let membership_service = store_value(membership_service);
    
    move || {
        // Segue il pattern di chat_view.rs: monitora ws_hook.messages per nuovi eventi
        create_effect(move |_| {
            let messages = ws_hook.messages.get();
            let membership_service = membership_service.get_value();
            
            // Cerca nuovi messaggi di tipo InvitationAccepted/InvitationRejected
            for ws_msg in messages {
                if let WebSocketMessage::Event { event, .. } = ws_msg {
                match event {
                    ServerEvent::Groups(GroupEvent::NewGroupMembership { group_id, new_membership_username }) => {
                        leptos::logging::log!("[MEMBERSHIP HANDLER] User '{}' joined group {}", new_membership_username, group_id);
                        
                        // Se l'evento riguarda il gruppo corrente, aggiorna il contatore
                        if let Some(current_id) = current_group_id.get() {
                            if group_id == current_id {
                                // Spawn async task per chiamare l'API di refresh
                                let membership_service = membership_service.clone();
                                let on_change = on_membership_change.get_value();
                                
                                spawn_local(async move {
                                    match membership_service.get_by_group_chat_id(&group_id.to_string()).await {
                                        Ok(members) => {
                                            leptos::logging::log!("[MEMBERSHIP HANDLER] Refreshed member count after new membership: {}", members.len());
                                            on_change(members.len() as i32);
                                        }
                                        Err(e) => {
                                            leptos::logging::error!("[MEMBERSHIP HANDLER] Failed to refresh member count: {:?}", e);
                                        }
                                    }
                                });
                            }
                        }
                    }
                    
                    ServerEvent::Groups(GroupEvent::LeftGroupMembership { group_id, left_membership_username }) => {
                        leptos::logging::log!("[MEMBERSHIP HANDLER] User '{}' left group {}", left_membership_username, group_id);
                        
                        // Se l'evento riguarda il gruppo corrente, aggiorna il contatore
                        if let Some(current_id) = current_group_id.get() {
                            if group_id == current_id {
                                // Spawn async task per chiamare l'API di refresh
                                let membership_service = membership_service.clone();
                                let on_change = on_membership_change.get_value();
                                
                                spawn_local(async move {
                                    match membership_service.get_by_group_chat_id(&group_id.to_string()).await {
                                        Ok(members) => {
                                            leptos::logging::log!("[MEMBERSHIP HANDLER] Refreshed member count after membership left: {}", members.len());
                                            on_change(members.len() as i32);
                                        }
                                        Err(e) => {
                                            leptos::logging::error!("[MEMBERSHIP HANDLER] Failed to refresh member count: {:?}", e);
                                        }
                                    }
                                });
                            }
                        }
                    }                        // Manteniamo la gestione degli eventi esistenti per compatibilità
                        ServerEvent::Groups(GroupEvent::Joined { user_id }) => {
                            leptos::logging::log!("[MEMBERSHIP HANDLER] User {} joined group", user_id);
                            
                            if let Some(current_id) = current_group_id.get() {
                                let membership_service = membership_service.clone();
                                let on_change = on_membership_change.get_value();
                                
                                spawn_local(async move {
                                    match membership_service.get_by_group_chat_id(&current_id.to_string()).await {
                                        Ok(members) => {
                                            leptos::logging::log!("[MEMBERSHIP HANDLER] Refreshed member count after join: {}", members.len());
                                            on_change(members.len() as i32);
                                        }
                                        Err(e) => {
                                            leptos::logging::error!("[MEMBERSHIP HANDLER] Failed to refresh member count after join: {:?}", e);
                                        }
                                    }
                                });
                            }
                        }
                        
                        ServerEvent::Groups(GroupEvent::Left { user_id }) => {
                            leptos::logging::log!("[MEMBERSHIP HANDLER] User {} left group", user_id);
                            
                            if let Some(current_id) = current_group_id.get() {
                                let membership_service = membership_service.clone();
                                let on_change = on_membership_change.get_value();
                                
                                spawn_local(async move {
                                    match membership_service.get_by_group_chat_id(&current_id.to_string()).await {
                                        Ok(members) => {
                                            leptos::logging::log!("[MEMBERSHIP HANDLER] Refreshed member count after leave: {}", members.len());
                                            on_change(members.len() as i32);
                                        }
                                        Err(e) => {
                                            leptos::logging::error!("[MEMBERSHIP HANDLER] Failed to refresh member count after leave: {:?}", e);
                                        }
                                    }
                                });
                            }
                        }
                        
                        _ => {} // Altri eventi non ci interessano per il membership
                    }
                }
            }
        });
    }
}

/// Versione semplificata che può essere integrata direttamente in componenti esistenti
/// Restituisce un effect che monitora gli eventi di membership (join/leave)
pub fn setup_membership_events_monitor(
    ws_hook: UseGroupMessageWs,
    _membership_service: GroupMembershipService,
    target_group_id: i32,
    on_user_joined: impl Fn(String) + Clone + 'static, // callback con username
    on_user_left: impl Fn(String) + Clone + 'static, // callback con username
) {
    let on_joined = store_value(on_user_joined);
    let on_left = store_value(on_user_left);
    
    create_effect(move |_| {
        let messages = ws_hook.messages.get();
        
        for ws_msg in messages {
            if let WebSocketMessage::Event { event, .. } = ws_msg {
                match event {
                    ServerEvent::Groups(GroupEvent::NewGroupMembership { group_id, new_membership_username }) => {
                        if group_id == target_group_id {
                            leptos::logging::log!("[MEMBERSHIP MONITOR] User '{}' joined group {}", new_membership_username, group_id);
                            on_joined.get_value()(new_membership_username.clone());
                        }
                    }
                    
                    ServerEvent::Groups(GroupEvent::LeftGroupMembership { group_id, left_membership_username }) => {
                        if group_id == target_group_id {
                            leptos::logging::log!("[MEMBERSHIP MONITOR] User '{}' left group {}", left_membership_username, group_id);
                            on_left.get_value()(left_membership_username.clone());
                        }
                    }
                    
                    _ => {}
                }
            }
        }
    });
}
