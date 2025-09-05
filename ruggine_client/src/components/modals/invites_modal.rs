use leptos::*;
use crate::types::Invitation;

#[component]
pub fn ShowInvitesModal(
    #[prop(into)] is_open: ReadSignal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] invites: ReadSignal<Vec<Invitation>>,
    #[prop(into, optional)] is_loading: Option<ReadSignal<bool>>,
    #[prop(into, optional)] on_accept: Option<Callback<i32>>,
    #[prop(into, optional)] on_reject: Option<Callback<i32>>,
) -> impl IntoView {
    let (is_visible, set_is_visible) = create_signal(false);
    let (is_animating_in, set_is_animating_in) = create_signal(false);

    // Gestione animazioni apertura/chiusura
    create_effect(move |_| {
        let open = is_open.get();
        if open {
            set_is_visible.set(true);
            set_timeout(move || set_is_animating_in.set(true), std::time::Duration::from_millis(10));
        } else {
            set_is_animating_in.set(false);
            set_timeout(move || set_is_visible.set(false), std::time::Duration::from_millis(250));
        }
    });

    let handle_close = move |_| {
        on_close.call(());
    };

    let handle_backdrop_click = move |e: web_sys::MouseEvent| {
        if let Some(target) = e.target() {
            if let Ok(element) = target.dyn_into::<web_sys::HtmlElement>() {
                if element.class_list().contains("modal-backdrop") {
                    handle_close(());
                }
            }
        }
    };

    view! {
        {move || if is_visible.get() {
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
                            let base_style = "width: 100%; max-width: 540px; margin: 0 20px; padding: 24px;";
                            if is_animating_in.get() {
                                format!("{}transform: scale(1) translateY(0px); opacity: 1; transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);", base_style)
                            } else {
                                format!("{}transform: scale(0.9) translateY(-20px); opacity: 0; transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);", base_style)
                            }
                        }
                    >
                        <div class="flex items-center justify-between mb-4">
                            <h2 class="text-xl font-semibold text-text-primary dark:text-text-primary-dark m-0">
                                "Inviti ricevuti"
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
                        <div class="mb-4 text-sm text-text-secondary dark:text-text-secondary-dark">
                            "Gestisci gli inviti ai gruppi che hai ricevuto. Puoi accettare o rifiutare ogni invito."
                        </div>
                        <div class="divide-y divide-border dark:divide-border-dark max-h-[340px] overflow-y-auto">
                            {move || {
                                let list = invites.get();
                                if list.is_empty() {
                                    view! {
                                        <div class="py-8 text-center text-text-secondary dark:text-text-secondary-dark">
                                            "Nessun invito ricevuto."
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {
                                        <ul class="space-y-0">
                                            {list.into_iter().map(|inv| {
                                                let id = inv.id;
                                                let group = inv.group_name.clone().unwrap_or_else(|| format!("Gruppo {}", inv.group_chat_id));
                                                let status = inv.status.clone();
                                                let sent_at = inv.created_at.format("%d/%m/%Y %H:%M").to_string();
                                                let is_pending = status.to_string() == "pending";
                                                view! {
                                                    <li class="py-4 flex items-center gap-4">
                                                        <div class="flex-1 min-w-0">
                                                            <div class="font-medium text-text-primary dark:text-text-primary-dark">{group}</div>
                                                            <div class="text-xs text-text-secondary dark:text-text-secondary-dark mt-1">Inviato il {sent_at}</div>
                                                        </div>
                                                        <div class="flex gap-2">
                                                            {if is_pending {
                                                                view! {
                                                                    <>
                                                                        {on_accept.as_ref().map(|cb| view! {
                                                                            <button class="px-3 py-1 text-xs rounded bg-green-500 text-white hover:bg-green-600 transition-colors" on:click=move |_| cb.call(id)>
                                                                                "Accetta"
                                                                            </button>
                                                                        })}
                                                                        {on_reject.as_ref().map(|cb| view! {
                                                                            <button class="px-3 py-1 text-xs rounded bg-red-500 text-white hover:bg-red-600 transition-colors" on:click=move |_| cb.call(id)>
                                                                                "Rifiuta"
                                                                            </button>
                                                                        })}
                                                                    </>
                                                                }.into_view()
                                                            } else {
                                                                view! {
                                                                    <span class="px-3 py-1 text-xs rounded bg-gray-300 dark:bg-gray-700 text-gray-600 dark:text-gray-300 cursor-default">
                                                                        {status.display_name()}
                                                                    </span>
                                                                }.into_view()
                                                            }}
                                                        </div>
                                                    </li>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </ul>
                                    }.into_view()
                                }
                            }}
                        </div>
                    </div>
                </div>
            }.into_view() else { view! { <></> }.into_view() }}
        }
    }
}
