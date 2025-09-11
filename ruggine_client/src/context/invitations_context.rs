use leptos::*;
use crate::types::Invitation;

/// Context for managing invitations globally
#[derive(Clone)]
pub struct InvitationsContext {
    pub invitations: RwSignal<Vec<Invitation>>,
    pub pending_count: RwSignal<usize>,
}

/// Provide the invitations context to the app
pub fn provide_invitations_context() -> (RwSignal<Vec<Invitation>>, RwSignal<usize>) {
    let invitations = create_rw_signal::<Vec<Invitation>>(Vec::new());
    let pending_count = create_rw_signal::<usize>(0);
    
    // Update pending count whenever invitations change
    {
        let pending_count = pending_count.clone();
        create_effect(move |_| {
            let invites = invitations.get();
            let count = invites.iter()
                .filter(|i| i.status.to_string() == "pending")
                .count();
            pending_count.set(count);
        });
    }
    
    provide_context(InvitationsContext {
        invitations: invitations.clone(),
        pending_count: pending_count.clone(),
    });
    
    (invitations, pending_count)
}

/// Access the invitations context
pub fn use_invitations_context() -> RwSignal<Vec<Invitation>> {
    use_context::<InvitationsContext>()
        .expect("InvitationsContext not found!")
        .invitations
}

/// Access the pending invitations count
pub fn use_pending_invitations_count_context() -> RwSignal<usize> {
    use_context::<InvitationsContext>()
        .expect("InvitationsContext not found!")
        .pending_count
}

/// Refresh invitations from the server
pub async fn refresh_invitations() -> Option<Vec<Invitation>> {
    use crate::api::client::ApiClient;
    use crate::config::constants::AppConstants;
    use crate::api::services::invitation::InvitationService;
    use crate::utils::storage::StorageService;
    use crate::utils::error_recovery::NetworkOperation;

    // Try to access the context; if not present, skip.
    let ctx = match use_context::<InvitationsContext>() {
        Some(c) => c,
        None => {
            leptos::logging::warn!("[INVITATIONS] InvitationsContext not found in refresh_invitations");
            return None;
        }
    };

    leptos::logging::log!("[INVITATIONS] refresh_invitations called");

    // Build client
    let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
    let storage = StorageService::new();
    if let Some(token) = storage.get_token() {
        http_client.set_auth_token(Some(token.token));
    }
    let invitation_service = InvitationService::new(http_client, storage);

    // Call the endpoint to get user invitations
    if let Some(invitations) = invitation_service.get_user_invitations()
        .with_auto_retry("refresh invitations from context").await {
        
        // Update the context
        let count = invitations.len();
        leptos::logging::log!("[INVITATIONS] refresh_invitations fetched {} invitations", count);
        ctx.invitations.set(invitations.clone());
        Some(invitations)
    } else {
        leptos::logging::warn!("[INVITATIONS] refresh_invitations failed to fetch invitations");
        None
    }
}

/// Add a new invitation to the context (called when WebSocket receives NewInvitation event)
pub async fn add_new_invitation(invitation_id: i32) {
    use crate::api::client::ApiClient;
    use crate::config::constants::AppConstants;
    use crate::api::services::invitation::InvitationService;
    use crate::utils::storage::StorageService;

    // Try to access the context; if not present, skip.
    let ctx = match use_context::<InvitationsContext>() {
        Some(c) => c,
        None => {
            leptos::logging::warn!("InvitationsContext not found, cannot add new invitation");
            return;
        }
    };

    // Build client to fetch the specific invitation
    let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
    let storage = StorageService::new();
    if let Some(token) = storage.get_token() {
        http_client.set_auth_token(Some(token.token));
    }
    let invitation_service = InvitationService::new(http_client, storage);

    // Fetch the specific invitation by ID
    match invitation_service.get_invitation_by_id(&invitation_id.to_string()).await {
        Ok(invitation) => {
            leptos::logging::log!("[INVITATIONS] Adding new invitation {} to context", invitation_id);
            ctx.invitations.update(|invites| {
                // Check if invitation already exists (avoid duplicates)
                if !invites.iter().any(|i| i.id == invitation.id) {
                    invites.push(invitation);
                }
            });
        }
        Err(e) => {
            leptos::logging::warn!("Failed to fetch invitation {}: {:?}", invitation_id, e);
            // Fallback: refresh all invitations
            let _ = refresh_invitations().await;
        }
    }
}
