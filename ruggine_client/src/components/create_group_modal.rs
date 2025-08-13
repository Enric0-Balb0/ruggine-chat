use leptos::*;
use leptos::wasm_bindgen::JsCast;
use crate::types::group::GroupChatCreateRequest;

#[component]
pub fn CreateGroupModal(
    #[prop(into)] is_open: ReadSignal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_create: Callback<GroupChatCreateRequest>,
    #[prop(into, optional)] is_loading: Option<ReadSignal<bool>>,
) -> impl IntoView {
    let (group_name, set_group_name) = create_signal(String::new());
    let (group_description, set_group_description) = create_signal(String::new());
    let (error_message, set_error_message) = create_signal(Option::<String>::None);

    // Use external loading state if provided, otherwise use internal state
    let (_internal_is_creating, _set_internal_is_creating) = create_signal(false);
    let is_creating = if let Some(external_loading) = is_loading {
        external_loading
    } else {
        _internal_is_creating.into()
    };

    // Animation states - animazione semplificata
    let (is_visible, set_is_visible) = create_signal(false);
    let (is_animating_in, set_is_animating_in) = create_signal(false);

    // Handle modal open/close - con animazioni di container
    create_effect(move |_| {
        let is_modal_open = is_open.get();
        
        if is_modal_open {
            set_is_visible.set(true);
            // Piccolo delay per permettere al DOM di renderizzare prima dell'animazione
            set_timeout(
                move || {
                    set_is_animating_in.set(true);
                },
                std::time::Duration::from_millis(10),
            );
        } else {
            set_is_animating_in.set(false);
            // Chiude con delay per l'animazione
            set_timeout(
                move || {
                    set_is_visible.set(false);
                },
                std::time::Duration::from_millis(250),
            );
        }
    });

    // Form validation
    let is_form_valid = create_memo(move |_| {
        let name = group_name.get();
        let desc = group_description.get();
        !name.trim().is_empty() && !desc.trim().is_empty() && name.len() >= 3 && name.len() <= 100
    });

    // Handle form submission
    let handle_submit = move |_| {
        if !is_form_valid.get() || is_creating.get() {
            return;
        }

        let name = group_name.get().trim().to_string();
        let description = group_description.get().trim().to_string();

        // Validate name length
        if name.len() < 3 {
            set_error_message.set(Some("Il nome del gruppo deve essere almeno di 3 caratteri".to_string()));
            return;
        }

        if name.len() > 100 {
            set_error_message.set(Some("Il nome del gruppo non può superare i 100 caratteri".to_string()));
            return;
        }

        // Validate description length
        if description.len() > 500 {
            set_error_message.set(Some("La descrizione non può superare i 500 caratteri".to_string()));
            return;
        }

        set_error_message.set(None);

        let create_request = GroupChatCreateRequest {
            name,
            description,
        };

        on_create.call(create_request);
    };

    // Handle close
    let handle_close = move |_| {
        // Reset form
        set_group_name.set(String::new());
        set_group_description.set(String::new());
        set_error_message.set(None);
        
        on_close.call(());
    };

    // Handle backdrop click
    let handle_backdrop_click = move |e: web_sys::MouseEvent| {
        // Only close if clicking on the backdrop (not the modal content)
        if let Some(target) = e.target() {
            if let Ok(element) = target.dyn_into::<web_sys::HtmlElement>() {
                if element.class_list().contains("modal-backdrop") {
                    handle_close(());
                }
            }
        }
    };

    // Character counts
    let name_count = create_memo(move |_| group_name.get().len());
    let desc_count = create_memo(move |_| group_description.get().len());

    view! {
        // Renderizza il modal solo se dovrebbe essere visibile
        {move || {
            if is_visible.get() {
                view! {
                    <div 
                        class="fixed inset-0 z-50 flex items-center justify-center modal-backdrop"
                        style=move || {
                            if is_animating_in.get() {
                                "background-color: rgba(0, 0, 0, 0.65); backdrop-filter: blur(4px); opacity: 1; transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);"
                            } else {
                                "background-color: rgba(0, 0, 0, 0); backdrop-filter: blur(0px); opacity: 0; transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);"
                            }
                        }
                        on:click=handle_backdrop_click
                    >
                        <div 
                            class="bg-white dark:bg-surface-dark shadow-2xl dark:shadow-black/50 border border-border dark:border-border-dark rounded-lg modal-container"
                            style=move || {
                                let base_style = "width: 100%; max-width: 440px; margin: 0 20px; padding: 24px;";
                                if is_animating_in.get() {
                                    format!("{}transform: scale(1) translateY(0px); opacity: 1; transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);", base_style)
                                } else {
                                    format!("{}transform: scale(0.9) translateY(-20px); opacity: 0; transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);", base_style)
                                }
                            }
                        >
                        // Header
                        <div class="flex items-center justify-between mb-4">
                            <h2 class="text-xl font-semibold text-text-primary dark:text-text-primary-dark m-0">
                                "Crea Nuovo Gruppo"
                            </h2>
                            <button
                                type="button"
                                class="bg-transparent border-none text-text-secondary dark:text-text-secondary-dark cursor-pointer p-2 rounded-lg transition-all duration-200 w-9 h-9 flex items-center justify-center hover:bg-red-50 hover:text-red-600 hover:scale-110 dark:hover:bg-red-900/20 dark:hover:text-red-400 group"
                                on:click=move |_| handle_close(())
                            >
                                <svg 
                                    class="w-5 h-5 transition-transform duration-300 group-hover:rotate-90"
                                    fill="none" 
                                    stroke="currentColor" 
                                    viewBox="0 0 24 24"
                                >
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                </svg>
                            </button>
                        </div>

                        // Descrizione della funzionalità
                        <div class="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
                            <div class="flex items-start gap-3">
                                <span class="text-blue-500 dark:text-blue-400 text-lg mt-0.5 animate-pulse">"💡"</span>
                                <div class="flex-1">
                                    <p class="m-0 text-sm text-blue-800 dark:text-blue-200 leading-relaxed">
                                        <strong class="font-medium">"Crea il tuo spazio di collaborazione!"</strong>
                                        <br/>
                                        "Inizia definendo nome e descrizione del gruppo. Potrai invitare i partecipanti e gestire i permessi successivamente."
                                    </p>
                                </div>
                            </div>
                        </div>

                        // Error message con animazione
                        {move || error_message.get().map(|msg| view! {
                            <div 
                                class="mb-5 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg"
                                style="transform: translateY(0px); opacity: 1; animation: slideInError 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);"
                            >
                                <p class="m-0 text-sm text-red-800 dark:text-red-200 flex items-center gap-2">
                                    <span class="animate-pulse text-base">"⚠️"</span>
                                    {msg}
                                </p>
                            </div>
                        })}

                        // Form
                        <form 
                            on:submit=move |e| {
                                e.prevent_default();
                                handle_submit(());
                            }
                        >
                            // Group Name Field
                            <div class="mb-4">
                                <label 
                                    for="group-name" 
                                    class="block text-sm font-semibold text-text-primary dark:text-text-primary-dark mb-2"
                                >
                                    "Nome del Gruppo"
                                </label>
                                <input
                                    type="text"
                                    id="group-name"
                                    class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark focus:border-transparent transition-colors hover:border-accent/50 dark:hover:border-accent-dark/50"
                                    placeholder="Inserisci il nome del gruppo..."
                                    prop:value=move || group_name.get()
                                    on:input=move |e| {
                                        set_group_name.set(event_target_value(&e));
                                        set_error_message.set(None);
                                    }
                                    maxlength="100"
                                    required
                                />
                                <div class="flex justify-between mt-2 text-xs">
                                    <span class="text-text-secondary dark:text-text-secondary-dark">
                                        "Minimo 3 caratteri"
                                    </span>
                                    <span class=move || {
                                        let count = name_count.get();
                                        let base_class = "font-medium transition-colors duration-200 ";
                                        if count > 90 {
                                            format!("{}text-red-500", base_class)
                                        } else if count > 70 {
                                            format!("{}text-orange-500", base_class)
                                        } else {
                                            format!("{}text-text-secondary dark:text-text-secondary-dark", base_class)
                                        }
                                    }>
                                        {move || format!("{}/100", name_count.get())}
                                    </span>
                                </div>
                            </div>

                            // Group Description Field
                            <div class="mb-4">
                                <label 
                                    for="group-description" 
                                    class="block text-sm font-semibold text-text-primary dark:text-text-primary-dark mb-2"
                                >
                                    "Descrizione"
                                </label>
                                <textarea
                                    id="group-description"
                                    class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark focus:border-transparent transition-colors resize-none hover:border-accent/50 dark:hover:border-accent-dark/50"
                                    rows="3"
                                    placeholder="Descrivi brevemente lo scopo del gruppo..."
                                    prop:value=move || group_description.get()
                                    on:input=move |e| {
                                        set_group_description.set(event_target_value(&e));
                                        set_error_message.set(None);
                                    }
                                    maxlength="500"
                                    required
                                >
                                </textarea>
                                <div class="flex justify-between mt-2 text-xs">
                                    <span class="text-text-secondary dark:text-text-secondary-dark">
                                        "Descrivi lo scopo del gruppo"
                                    </span>
                                    <span class=move || {
                                        let count = desc_count.get();
                                        let base_class = "font-medium transition-colors duration-200 ";
                                        if count > 450 {
                                            format!("{}text-red-500", base_class)
                                        } else if count > 350 {
                                            format!("{}text-orange-500", base_class)
                                        } else {
                                            format!("{}text-text-secondary dark:text-text-secondary-dark", base_class)
                                        }
                                    }>
                                        {move || format!("{}/500", desc_count.get())}
                                    </span>
                                </div>
                            </div>

                            // Action Buttons
                            <div class="flex gap-3 mt-6">
                                <button
                                    type="button"
                                    class="flex-1 px-4 py-2.5 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark font-medium cursor-pointer transition-colors hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
                                    on:click=move |_| handle_close(())
                                    disabled=move || is_creating.get()
                                >
                                    "Annulla"
                                </button>
                                <button
                                    type="submit"
                                    class=move || {
                                        let base_classes = "flex-1 px-4 py-2.5 text-sm rounded-md font-medium cursor-pointer transition-colors border-none flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed";
                                        if is_form_valid.get() && !is_creating.get() {
                                            format!("{} bg-brand-primary dark:bg-brand-primary-dark hover:bg-brand-primary-light dark:hover:bg-brand-primary text-white", base_classes)
                                        } else {
                                            format!("{} bg-gray-300 dark:bg-gray-600 text-gray-500 dark:text-gray-400", base_classes)
                                        }
                                    }
                                    disabled=move || !is_form_valid.get() || is_creating.get()
                                >
                                    {move || if is_creating.get() {
                                        view! {
                                            "Creando..."
                                        }.into_view()
                                    } else {
                                        view! { 
                                            "Crea Gruppo"
                                        }.into_view()
                                    }}
                                </button>
                            </div>
                        </form>
                        
                        // CSS personalizzato per animazioni semplici
                        <style>
                            "
                            @keyframes slideInError {
                                0% {
                                    transform: translateY(-10px) scale(0.95);
                                    opacity: 0;
                                }
                                50% {
                                    transform: translateY(2px) scale(1.02);
                                }
                                100% {
                                    transform: translateY(0px) scale(1);
                                    opacity: 1;
                                }
                            }
                            
                            @keyframes spin {
                                from {
                                    transform: rotate(0deg);
                                }
                                to {
                                    transform: rotate(360deg);
                                }
                            }
                            
                            .modal-backdrop {
                                will-change: opacity, backdrop-filter;
                            }
                            
                            .modal-container {
                                will-change: transform, opacity;
                            }
                            
                            /* Smooth scrollbar per textarea */
                            textarea::-webkit-scrollbar {
                                width: 6px;
                            }
                            
                            textarea::-webkit-scrollbar-track {
                                background: rgba(0, 0, 0, 0.05);
                                border-radius: 3px;
                            }
                            
                            textarea::-webkit-scrollbar-thumb {
                                background: rgba(0, 0, 0, 0.2);
                                border-radius: 3px;
                                transition: background 0.2s ease;
                            }
                            
                            textarea::-webkit-scrollbar-thumb:hover {
                                background: rgba(0, 0, 0, 0.3);
                            }
                            
                            /* Dark mode scrollbar */
                            .dark textarea::-webkit-scrollbar-track {
                                background: rgba(255, 255, 255, 0.05);
                            }
                            
                            .dark textarea::-webkit-scrollbar-thumb {
                                background: rgba(255, 255, 255, 0.2);
                            }
                            
                            .dark textarea::-webkit-scrollbar-thumb:hover {
                                background: rgba(255, 255, 255, 0.3);
                            }
                            "
                        </style>
                    </div>
                </div>
                }.into_view()
            } else {
                view! { <></> }.into_view()
            }
        }}
    }
}
