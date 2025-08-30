use leptos::*;
use leptos::ev::MouseEvent;
use crate::components::{AppNavbar, LucideIcon};
use crate::components::ui::icons::IconSize;
use crate::api::services::CpuUsageLogService;
use crate::api::client::ApiClient;
use crate::utils::StorageService;
use crate::config::constants::AppConstants;
use crate::types::cpu_usage_log::CpuUsageLogReadDto;
use web_sys;

#[component]
pub fn CpuLogsPage() -> impl IntoView {
    // Api client + service: apply stored token (no refresh)
    let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
    let storage = StorageService::new();
    if let Some(token_response) = storage.get_token() {
        http_client.set_auth_token(Some(token_response.token));
    }
    let service = CpuUsageLogService::new(http_client, storage);

    // Signals
    let (logs, set_logs) = create_signal::<Vec<CpuUsageLogReadDto>>(vec![]);
    let (next_cursor, set_next_cursor) = create_signal::<Option<String>>(None);
    let (has_more, set_has_more) = create_signal(false);
    let page_size = 20;

    // show_load_more considers both server has_more and presence of a cursor
    let show_load_more = create_memo(move |_| has_more.get() || next_cursor.get().is_some());

    let format_ts = move |ts: &str| {
        let mut s = ts.replace("T", " ");
        if let Some(idx) = s.find('Z') { s.truncate(idx); }
        else if let Some(pos) = s.find('+') { s.truncate(pos); }
        s
    };

    // initial page load
    {
        let service = service.clone();
        let set_logs = set_logs.clone();
        let set_next_cursor = set_next_cursor.clone();
        let set_has_more = set_has_more.clone();
        spawn_local(async move {
            if let Ok(page) = service.find_paginated(None, Some(page_size)).await {
                set_logs.set(page.items.clone());
                set_next_cursor.set(page.next_cursor.clone());
                set_has_more.set(page.has_more);
            }
        });
    }

    // load more handler (cursor-based)
    let load_more = {
        let service = service.clone();
        let next_cursor = next_cursor.clone();
        let set_logs = set_logs.clone();
        let set_next_cursor = set_next_cursor.clone();
        let set_has_more = set_has_more.clone();
        let logs_signal = logs.clone();
        Callback::new(move |_: MouseEvent| {
            let service = service.clone();
            let next = next_cursor.get();
            let set_logs = set_logs.clone();
            let set_next_cursor = set_next_cursor.clone();
            let set_has_more = set_has_more.clone();
            let mut combined = logs_signal.get();
            spawn_local(async move {
                if let Ok(page) = service.find_paginated(next.clone(), Some(page_size)).await {
                    combined.extend(page.items.clone());
                    set_logs.set(combined);
                    set_next_cursor.set(page.next_cursor.clone());
                    set_has_more.set(page.has_more);
                }
            });
        })
    };

    view! {
        <div class="flex flex-col min-h-screen bg-white dark:bg-surface-dark">
            <AppNavbar />

            <main class="flex-1 flex flex-col items-center p-4 min-h-0 pb-24">
                <div class="w-full max-w-6xl flex flex-col gap-4 h-full" style="min-height:0;">
                    <div class="w-full rounded border border-gray-200 dark:border-border-dark bg-gray-50 dark:bg-surface-dark p-4">
                        <div class="flex items-center gap-3">
                            <button class="p-2 rounded bg-white/0 dark:bg-transparent hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors" on:click={move |_| {
                                if let Some(win) = web_sys::window() {
                                    let _ = win.history().and_then(|h| h.back());
                                }
                            }}>
                                <LucideIcon name="arrow-left" size=IconSize::SMALL class="text-gray-900 dark:text-white" />
                            </button>
                            <h1 class="text-2xl font-semibold text-gray-900 dark:text-white">"CPU Usage Logs"</h1>
                        </div>
                    </div>

                    <div class="w-full rounded border border-gray-200 dark:border-border-dark bg-gray-50 dark:bg-surface-dark p-4 flex flex-col flex-1 min-h-0" style="min-height:0;">
                        <div class="flex-1 overflow-y-auto min-h-0 max-h-[calc(100vh-18rem)]">
                            <table class="min-w-full table-auto divide-y divide-gray-200 dark:divide-border-dark">
                                <thead>
                                    <tr>
                                        <th class="sticky top-0 z-10 px-4 py-2 bg-gray-100 dark:bg-surface-dark text-left text-sm text-gray-600 dark:text-text-secondary-dark">"Timestamp"</th>
                                        <th class="sticky top-0 z-10 px-4 py-2 bg-gray-100 dark:bg-surface-dark text-left text-sm text-gray-600 dark:text-text-secondary-dark">"CPU %"</th>
                                        <th class="sticky top-0 z-10 px-4 py-2 bg-gray-100 dark:bg-surface-dark text-left text-sm text-gray-600 dark:text-text-secondary-dark">"ID"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <For each=move || logs.get() key=|item: &CpuUsageLogReadDto| item.id children=move |item: CpuUsageLogReadDto| {
                                        view! {
                                            <tr class="odd:bg-white even:bg-gray-50 dark:odd:bg-surface-dark dark:even:bg-surface-dark border-b border-gray-100 dark:border-border-dark">
                                                <td class="px-4 py-3 text-sm text-gray-700 dark:text-text-primary-dark">{format_ts(&item.timestamp)}</td>
                                                <td class="px-4 py-3 text-sm font-medium text-gray-900 dark:text-white">{format!("{}%", item.cpu_usage_percent)}</td>
                                                <td class="px-4 py-3 text-xs text-gray-500 dark:text-text-secondary-dark">{item.id}</td>
                                            </tr>
                                        }
                                    } />
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </main>

            { move || {
                if show_load_more.get() {
                    view! {
                        <div>
                            <div class="fixed bottom-6 left-1/2 transform -translate-x-1/2 z-50">
                                <button class="px-6 py-3 bg-brand-primary text-white rounded-full shadow-lg hover:brightness-95 transition" on:click={let handler = load_more.clone(); move |e| handler.call(e)}>
                                    "Carica altri"
                                </button>
                            </div>
                            <footer class="w-full py-6 flex justify-center invisible">
                                <div></div>
                            </footer>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <footer class="w-full py-6 flex justify-center">
                            <div class="text-sm text-gray-700 dark:text-gray-200 bg-white/80 dark:bg-black/60 px-4 py-2 rounded shadow backdrop-blur-sm">"Nessun altro log"</div>
                        </footer>
                    }.into_view()
                }
            } }
        </div>
    }
}
