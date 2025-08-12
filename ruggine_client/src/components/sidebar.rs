use leptos::*;
use crate::components::{GroupItem, CreateGroupButton};

/// Sidebar component for the main app layout
#[component]
pub fn Sidebar() -> impl IntoView {
    // TODO: Sostituire con dati reali dal servizio
    let groups = vec![
        ("Team Alpha", false),
        ("Progetto Beta", true), // Gruppo attivo
        ("Marketing", false),
        ("Sviluppo Frontend", false),
        ("Design Team", false),
    ];

    view! {
        <div class="w-[280px] bg-bg-sidebar dark:bg-bg-sidebar-dark border-r border-border dark:border-border-dark flex flex-col">
            // Teams Section
            <div class="flex-1 p-4">
                <div class="text-xs font-semibold text-text-secondary dark:text-text-secondary-dark uppercase tracking-wide mb-3">
                    "Team e Gruppi"
                </div>
                
                // Groups List
                <div class="space-y-1">
                    {groups.into_iter().map(|(name, is_active)| {
                        view! {
                            <GroupItem name=name.to_string() is_active=is_active />
                        }
                    }).collect::<Vec<_>>()}
                    
                    // Separatore sottile prima del pulsante di creazione
                    <div class="h-px bg-border dark:bg-border-dark my-2"></div>
                    
                    // Create new group button
                    <CreateGroupButton />
                </div>
            </div>
        </div>
    }
}
