use leptos::*;
use crate::components::{GroupItem, CreateGroupButton};
use crate::hooks::{use_groups_context, use_groups_list, use_groups_loading, use_groups_error};

/// Sidebar component for the main app layout
#[component]
pub fn Sidebar(
    #[prop(into)] on_create_group_click: Callback<()>,
) -> impl IntoView {
    // Use groups context from provider
    let groups_context = use_groups_context();
    let groups_hook = groups_context.groups_hook;
    let groups_list = use_groups_list(&groups_hook);
    let is_loading = use_groups_loading(&groups_hook);
    let error = use_groups_error(&groups_hook);

    // Load groups on mount - only if we have auth
    create_effect(move |_| {
        // Simple check - try to get token from localStorage to see if we should load groups
        if let Some(window) = web_sys::window() {
            if let Some(storage) = window.local_storage().ok().flatten() {
                if let Ok(Some(_token)) = storage.get_item("ruggine_auth_token") {
                    logging::log!("Token found, fetching groups...");
                    groups_hook.refresh_groups.dispatch(());
                } else {
                    logging::log!("No token found, skipping groups fetch");
                }
            }
        }
    });

    // Handle group selection
    let set_active_group = groups_hook.set_active_group;
    let active_group_id = groups_hook.active_group_id;
    
    let handle_group_click = move |group_id: i32| {
        set_active_group.set(Some(group_id));
        logging::log!("Selected group: {}", group_id);
    };

    // Handle create group button click
    let handle_create_click = move |_| {
        on_create_group_click.call(());
    };

    view! {
        <div class="w-[280px] bg-bg-sidebar dark:bg-bg-sidebar-dark border-r border-border dark:border-border-dark flex flex-col">
            // Teams Section
            <div class="flex-1 p-4">
                <div class="text-xs font-semibold text-text-secondary dark:text-text-secondary-dark uppercase tracking-wide mb-3">
                    "Team e Gruppi"
                </div>
                
                // Groups List
                <div class="space-y-1">
                    {move || {
                        if let Some(_error_msg) = error.get() {
                            view! {
                                <div class="flex flex-col items-center justify-center py-4 space-y-2">
                                    <div class="text-sm text-red-500">
                                        "Errore nel caricamento"
                                    </div>
                                    <button 
                                        class="text-xs text-brand-primary-light hover:underline"
                                        on:click=move |_| groups_hook.refresh_groups.dispatch(())
                                    >
                                        "Riprova"
                                    </button>
                                </div>
                            }.into_view()
                        } else {
                            let groups = groups_list.get();
                            if groups.is_empty() && !is_loading.get() {
                                view! {
                                    <div class="flex items-center justify-center py-4">
                                        <div class="text-sm text-text-secondary dark:text-text-secondary-dark">
                                            "Nessun gruppo trovato"
                                        </div>
                                    </div>
                                }.into_view()
                            } else {
                                view! {
                                    <div class="space-y-1">
                                        {groups.into_iter().enumerate().map(|(index, group_data)| {
                                            let is_active = active_group_id.get() == Some(group_data.membership.group_chat_id);
                                            view! {
                                                <div 
                                                    class="animate-fade-in-up"
                                                    style=format!("animation-delay: {}ms", index * 100)
                                                >
                                                    <GroupItem 
                                                        group_data=group_data
                                                        is_active=is_active 
                                                        on_click=handle_group_click
                                                    />
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_view()
                            }
                        }
                    }}
                    
                    // Separatore sottile prima del pulsante di creazione
                    <div class="h-px bg-border dark:bg-border-dark my-2"></div>
                    
                    // Create new group button
                    <CreateGroupButton on_create_click=handle_create_click />
                </div>
            </div>
        </div>
    }
}
