use leptos::*;
use crate::components::{AppNavbar, Sidebar};

#[component]
pub fn HomePage() -> impl IntoView {

    view! {
        <div class="h-screen w-screen overflow-hidden bg-white dark:bg-bg-main-dark flex">
            // Left Sidebar - Occupa tutto il lato sinistro dall'alto in basso
            <Sidebar />

            // Main Content Area - Include navbar + content
            <div class="flex-1 flex flex-col">
                // Top Header/Navbar - Solo nella parte destra
                <AppNavbar />

                // Main Content Area
                <div class="flex-1 bg-white dark:bg-bg-main-dark flex items-center justify-center">
                    <div class="text-center max-w-md">
                        <div class="w-16 h-16 bg-brand-primary-light rounded-full flex items-center justify-center mx-auto mb-4">
                            <span class="text-white text-xl font-bold">"💬"</span>
                        </div>
                        <h1 class="text-2xl font-bold text-text-primary dark:text-text-primary-dark mb-4">
                            "Benvenuto in Ruggine"
                        </h1>
                        <p class="text-text-secondary dark:text-text-secondary-dark mb-6">
                            "Seleziona un team dalla sidebar per iniziare una conversazione, oppure crea un nuovo team."
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}
