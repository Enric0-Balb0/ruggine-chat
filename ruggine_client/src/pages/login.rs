// Simple login page for testing
use crate::api::RuggineApiClient;
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

    // Create API client using the proper facade
    let api_client = RuggineApiClient::new();
    let api_client_clone1 = api_client.clone();
    let api_client_clone2 = api_client.clone();
    let api_client_clone3 = api_client.clone();

    // Login action
    let handle_login = move |_| {
        let api_client = api_client_clone1.clone();
        let email_val = email.get();
        let password_val = password.get();
        
        spawn_local(async move {
            set_loading.set(true);
            set_error_message.set(None);
            set_success_message.set(None);

            match api_client.auth_service.login(email_val, password_val).await {
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
        <div class="min-h-screen flex items-center justify-center bg-bg-main py-12 px-4 sm:px-6 lg:px-8">
            <div class="max-w-md w-full space-y-8">
                <div class="text-center">
                    <h1 class="text-3xl font-bold text-brand-primary mb-2">"Ruggine Chat"</h1>
                    <h2 class="text-lg text-brand-secondary-light">"Sign in to your account"</h2>
                </div>
                
                <div class="card">
                    // Email input
                    <div class="mb-4">
                        <label for="email" class="block text-sm font-medium text-brand-secondary mb-2">"Email address"</label>
                        <input
                            type="email"
                            id="email"
                            placeholder="Enter your email"
                            class="input-field"
                            prop:value=email
                            on:input=move |ev| {
                                set_email.set(event_target_value(&ev));
                            }
                            prop:disabled=loading
                        />
                    </div>

                    // Password input
                    <div class="mb-6">
                        <label for="password" class="block text-sm font-medium text-brand-secondary mb-2">"Password"</label>
                        <input
                            type="password"
                            id="password"
                            placeholder="Enter your password"
                            class="input-field"
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
                        class={move || if loading.get() { 
                            "w-full bg-blue-400 text-white font-medium py-2 px-4 rounded-lg cursor-not-allowed" 
                        } else { 
                            "btn-primary w-full" 
                        }}
                    >
                        {move || if loading.get() { "Signing in..." } else { "Sign in" }}
                    </button>

                    // Error message
                    {move || {
                        error_message.get().map(|msg| {
                            view! {
                                <div class="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                                    <p class="text-sm text-red-700">{msg}</p>
                                </div>
                            }
                        })
                    }}

                    // Success message
                    {move || {
                        success_message.get().map(|msg| {
                            view! {
                                <div class="mt-4 p-3 bg-green-50 border border-green-200 rounded-lg">
                                    <p class="text-sm text-green-700">{msg}</p>
                                </div>
                            }
                        })
                    }}
                </div>

                // Auth status
                <div class="card">
                    <h3 class="text-lg font-semibold text-brand-secondary mb-3">"Current Auth Status"</h3>
                    <div class="space-y-2 text-sm">
                        <p class="flex justify-between">
                            <span class="text-brand-secondary-light">"Authenticated:"</span>
                            <span class={move || if api_client_clone2.is_authenticated() { "text-status-success font-medium" } else { "text-status-danger font-medium" }}>
                                {move || api_client_clone3.is_authenticated().to_string()}
                            </span>
                        </p>
                        <p class="flex justify-between">
                            <span class="text-brand-secondary-light">"User:"</span>
                            <span class="text-brand-secondary font-medium">
                                {move || {
                                    api_client.get_current_user()
                                        .map(|u| format!("{} ({})", u.full_name, u.email))
                                        .unwrap_or_else(|| "None".to_string())
                                }}
                            </span>
                        </p>
                    </div>
                    
                    // TODO: Add logout functionality later
                </div>

                // Test credentials info
                <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
                    <h3 class="text-sm font-semibold text-blue-900 mb-2">"Test Credentials"</h3>
                    <div class="space-y-1 text-sm text-blue-700">
                        <p><span class="font-medium">"Email:"</span> " test@example.com"</p>
                        <p><span class="font-medium">"Password:"</span> " password123"</p>
                        <p class="text-blue-600 italic text-xs mt-2">"Note: This will call the real API endpoints"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}
