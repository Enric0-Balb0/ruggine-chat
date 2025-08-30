use leptos::*;
use leptos_router::*;
use crate::pages::{LandingPage, RegisterPage, HomePage, ProfilePage};
use crate::components::AppLayout;
use super::guards::AuthGuard;
use super::login_guard::PublicGuard;

#[component]
pub fn AppRouter() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                // Landing page with login - protetto per utenti già autenticati
                <Route path="/login" view=|| {
                    view! {
                        <PublicGuard>
                            <LandingPage />
                        </PublicGuard>
                    }
                } />
                
                // Registration page - protetto per utenti già autenticati
                <Route path="/register" view=|| {
                    view! {
                        <PublicGuard>
                            <RegisterPage />
                        </PublicGuard>
                    }
                } />
                
                // Protected app routes
                <Route path="/" view=|| {
                    view! { 
                        <AuthGuard>
                            <AppLayout>
                                <HomePage />
                            </AppLayout>
                        </AuthGuard>
                    }
                } />
                <Route path="/profile" view=|| {
                    view! {
                        <AuthGuard>
                            <AppLayout>
                                <ProfilePage />
                            </AppLayout>
                        </AuthGuard>
                    }
                } />
                <Route path="/admin/cpu-logs" view=|| {
                    view! {
                        <AuthGuard>
                            <AppLayout>
                                <crate::pages::CpuLogsPage />
                            </AppLayout>
                        </AuthGuard>
                    }
                } />
                
                // Fallback route - redirect to home which will handle auth
                <Route path="/*any" view=|| {
                    let navigate = use_navigate();
                    create_effect(move |_| {
                        navigate("/", Default::default());
                    });
                    view! { <div class="min-h-screen flex items-center justify-center">
                        <div class="text-center">
                            <p class="text-brand-secondary-light">"Reindirizzamento..."</p>
                        </div>
                    </div> }
                } />
            </Routes>
        </Router>
    }
}
