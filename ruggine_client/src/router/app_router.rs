use leptos::*;
use leptos_router::*;
use crate::pages::{LandingPage, RegisterPage, HomePage};
use crate::components::AppLayout;
use super::guards::AuthGuard;

#[component]
pub fn AppRouter() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                // Landing page with login
                <Route path="/login" view=LandingPage />
                
                // Registration page
                <Route path="/register" view=RegisterPage />
                
                // Protected app routes
                <Route path="/" view=|| view! { 
                    <AuthGuard>
                        <AppLayout>
                            <HomePage />
                        </AppLayout>
                    </AuthGuard>
                } />
                
                // Fallback route - redirect to home which will handle auth
                <Route path="/*any" view=|| {
                    let navigate = use_navigate();
                    create_effect(move |_| {
                        navigate("/", Default::default());
                    });
                    view! { <div class="min-h-screen flex items-center justify-center">
                        <div class="text-center">
                            <p class="text-[#605e5c]">"Reindirizzamento..."</p>
                        </div>
                    </div> }
                } />
            </Routes>
        </Router>
    }
}
