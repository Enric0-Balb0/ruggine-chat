// Simple login page for testing
use crate::services::auth_service::AuthService;
use leptos::*;

/// Login page component
#[component]
pub fn LoginPage() -> impl IntoView {
    // Form state
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (loading, set_loading) = create_signal(false);
    let (error_message, set_error_message) = create_signal(None::<String>);
    let (success_message, set_success_message) = create_signal(None::<String>);

    // Create auth service
    let http_client = crate::http::client::ApiClient::new("http://localhost:8080");
    let storage_service = crate::services::storage_service::StorageService::new();
    let auth_service = AuthService::new(http_client, storage_service);
    let auth_service_clone1 = auth_service.clone();
    let auth_service_clone2 = auth_service.clone();

    // Login action
    let handle_login = move |_| {
        let auth_service = auth_service_clone1.clone();
        let email_val = email.get();
        let password_val = password.get();
        
        spawn_local(async move {
            set_loading.set(true);
            set_error_message.set(None);
            set_success_message.set(None);

            match auth_service.login(email_val, password_val).await {
                Ok(user_profile) => {
                    set_success_message.set(Some(format!(
                        "Login successful! Welcome, {}",
                        user_profile.full_name
                    )));
                    // Clear form
                    set_email.set(String::new());
                    set_password.set(String::new());
                }
                Err(e) => {
                    set_error_message.set(Some(format!("Login failed: {}", e)));
                }
            }
            
            set_loading.set(false);
        });
    };

    view! {
        <div class="login-container">
            <h1>"Ruggine Client - Login Test"</h1>
            
            <div class="login-form">
                <h2>"Login"</h2>
                
                // Email input
                <div class="form-group">
                    <label for="email">"Email:"</label>
                    <input
                        type="email"
                        id="email"
                        placeholder="Enter your email"
                        prop:value=email
                        on:input=move |ev| {
                            set_email.set(event_target_value(&ev));
                        }
                        prop:disabled=loading
                    />
                </div>

                // Password input
                <div class="form-group">
                    <label for="password">"Password:"</label>
                    <input
                        type="password"
                        id="password"
                        placeholder="Enter your password"
                        prop:value=password
                        on:input=move |ev| {
                            set_password.set(event_target_value(&ev));
                        }
                        prop:disabled=loading
                    />
                </div>

                // Submit button
                <button
                    on:click=handle_login
                    prop:disabled=loading
                    class="login-button"
                >
                    {move || if loading.get() { "Logging in..." } else { "Login" }}
                </button>

                // Error message
                {move || {
                    error_message.get().map(|msg| {
                        view! {
                            <div class="error-message">
                                {msg}
                            </div>
                        }
                    })
                }}

                // Success message
                {move || {
                    success_message.get().map(|msg| {
                        view! {
                            <div class="success-message">
                                {msg}
                            </div>
                        }
                    })
                }}

                // Auth status
                <div class="auth-status">
                    <h3>"Current Auth Status:"</h3>
                    <p>"Authenticated: " {move || auth_service_clone2.is_authenticated().to_string()}</p>
                    <p>"User: " {move || {
                        auth_service.get_current_user()
                            .map(|u| format!("{} ({})", u.full_name, u.email))
                            .unwrap_or_else(|| "None".to_string())
                    }}</p>
                </div>

                // Test credentials info
                <div class="test-info">
                    <h3>"Test Credentials:"</h3>
                    <p><strong>"Email:"</strong> " test@example.com"</p>
                    <p><strong>"Password:"</strong> " password123"</p>
                    <p><em>"Note: This will call the real API endpoints"</em></p>
                </div>
            </div>
        </div>
    }
}
