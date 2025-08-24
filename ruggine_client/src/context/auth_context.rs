use leptos::*;

#[derive(Clone)]
pub struct AuthContext {
    pub token: RwSignal<Option<String>>,
}

pub fn provide_auth_context() -> AuthContext {
    use crate::utils::storage::StorageService;
    let token = create_rw_signal(None);

    // Recupera il token da localStorage se presente
    let storage = StorageService::new();
    if let Some(token_response) = storage.get_token() {
        token.set(Some(token_response.token));
    }

    let ctx = AuthContext { token };
    provide_context(ctx.clone());
    ctx
}

pub fn use_auth_context() -> AuthContext {
    use_context::<AuthContext>().expect("AuthContext not found. Did you forget to call provide_auth_context()?")
}
