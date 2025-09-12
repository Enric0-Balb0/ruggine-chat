// ESEMPIO DI INTEGRAZIONE nel componente OnlineUsersCounter esistente
// Questo è un esempio di come integrare il nuovo handler nel contatore online

use leptos::*;
use crate::components::ui::group_membership_ws_handler::setup_membership_events_monitor;
use crate::hooks::use_group_message_ws::UseGroupMessageWs;
use crate::api::services::membership::GroupMembershipService;

// All'interno del componente OnlineUsersCounter, dopo aver configurato il WebSocket hook:

pub fn integrate_membership_monitoring(
    ws_hook: UseGroupMessageWs,
    membership_service: GroupMembershipService,
    online_count: WriteSignal<i32>,
    member_count: WriteSignal<i32>, // Se presente un signal per il contatore membri
) {
    // Configuriamo il monitoraggio per tutti i gruppi (senza target specifico)
    // In alternativa si può passare un group_id specifico se disponibile
    
    create_effect(move |_| {
        let messages = ws_hook.messages.get();
        
        for ws_msg in messages.iter() {
            if let crate::types::message_ws::WebSocketMessage::Event { event, .. } = ws_msg {
                match event {
                    crate::types::message_ws::ServerEvent::Groups(
                        crate::types::message_ws::GroupEvent::NewGroupMembership { group_id, new_membership_username }
                    ) => {
                        leptos::logging::log!("[ONLINE COUNTER] User '{}' joined group {}", new_membership_username, group_id);
                        
                        // Refresh del contatore membri per il gruppo specifico
                        let membership_service = membership_service.clone();
                        spawn_local(async move {
                            // Qui possiamo chiamare l'API per aggiornare sia il contatore membri che quello online
                            // Esempio: refresh del contatore online globale
                            match membership_service.find_connected_users_and_online().await {
                                Ok(users) => {
                                    leptos::logging::log!("[ONLINE COUNTER] Refreshed online count after invitation acceptance: {}", users.len());
                                    online_count.set(users.len() as i32);
                                }
                                Err(e) => {
                                    leptos::logging::error!("[ONLINE COUNTER] Failed to refresh online count: {:?}", e);
                                }
                            }
                            
                            // Se abbiamo anche un contatore membri per il gruppo specifico:
                            if let Ok(members) = membership_service.get_by_group_chat_id(&group_chat_id.to_string()).await {
                                member_count.set(members.len() as i32);
                            }
                        });
                    }
                    
                    crate::types::message_ws::ServerEvent::Groups(
                        crate::types::message_ws::GroupEvent::InvitationRejected { user_id, group_chat_id }
                    ) => {
                        leptos::logging::log!("[ONLINE COUNTER] User {} rejected invitation to group {}", user_id, group_chat_id);
                        // Il rifiuto non cambia i contatori, ma possiamo loggare o fare altre azioni
                    }
                    
                    _ => {}
                }
            }
        }
    });
}

// ESEMPIO DI USO nell'attuale chat_view.rs per integrare con il contatore membri esistente:

pub fn extend_chat_view_membership_monitoring(
    ws_hook: UseGroupMessageWs,
    membership_service: GroupMembershipService,
    current_group_id: ReadSignal<Option<i32>>,
    set_current_member_count: WriteSignal<i32>,
) {
    create_effect(move |_| {
        let messages = ws_hook.messages.get();
        
        for ws_msg in messages.iter() {
            if let crate::types::message_ws::WebSocketMessage::Event { event, .. } = ws_msg {
                match event {
                    // Nuovi eventi di accettazione/rifiuto inviti
                    crate::types::message_ws::ServerEvent::Groups(
                        crate::types::message_ws::GroupEvent::InvitationAccepted { user_id, group_chat_id }
                    ) => {
                        if let Some(current_id) = current_group_id.get() {
                            if *group_chat_id == current_id {
                                leptos::logging::log!("[CHAT VIEW] User {} accepted invitation to current group", user_id);
                                
                                // Refresh del contatore membri seguendo il pattern esistente
                                let membership_service = membership_service.clone();
                                spawn_local(async move {
                                    match membership_service.get_by_group_chat_id(&group_chat_id.to_string()).await {
                                        Ok(memberships) => {
                                            set_current_member_count.set(memberships.len() as i32);
                                            leptos::logging::log!("[CHAT VIEW] Updated member count after invitation acceptance: {}", memberships.len());
                                        }
                                        Err(e) => {
                                            leptos::logging::error!("[CHAT VIEW] Failed to refresh member count: {:?}", e);
                                        }
                                    }
                                });
                            }
                        }
                    }
                    
                    crate::types::message_ws::ServerEvent::Groups(
                        crate::types::message_ws::GroupEvent::InvitationRejected { user_id, group_chat_id }
                    ) => {
                        leptos::logging::log!("[CHAT VIEW] User {} rejected invitation to group {}", user_id, group_chat_id);
                        // Il rifiuto non cambia il numero di membri attuali
                    }
                    
                    // Manteniamo la gestione esistente per Joined/Left
                    crate::types::message_ws::ServerEvent::Groups(
                        crate::types::message_ws::GroupEvent::Joined { .. } | 
                        crate::types::message_ws::GroupEvent::Left { .. }
                    ) => {
                        // Il codice esistente in chat_view.rs già gestisce questi eventi
                        // Questo è solo per riferimento
                    }
                    
                    _ => {}
                }
            }
        }
    });
}
