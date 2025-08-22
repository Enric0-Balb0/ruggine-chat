
use leptos::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::hooks::GroupMembershipWithDetails;

#[component]
pub fn GroupItem(
    #[prop(into)] group_data: GroupMembershipWithDetails,
    #[prop(into, default = false)] is_active: bool,
    #[prop(into, optional)] on_click: Option<Callback<i32>>,
) -> impl IntoView {
    let group_name = group_data.group_name();
    let membership = group_data.membership.clone();
    
    // Genera un colore basato sul nome del gruppo
    let bg_color = generate_group_color(&group_name);
    
    // Genera le iniziali del gruppo (prime 2 lettere o prime lettere di 2 parole)
    let initials = get_group_initials(&group_name);

    let active_class = if is_active {
        "bg-bg-main dark:bg-bg-main-dark border-l-2 border-brand-primary-light"
    } else {
        "hover:bg-bg-main dark:hover:bg-bg-main-dark"
    };

    // Handle click
    let group_id = membership.group_chat_id;
    let handle_click = move |_| {
        if let Some(on_click) = on_click {
            on_click.call(group_id);
        }
    };

    view! {
        <div 
            class=format!("flex items-center p-2 rounded cursor-pointer transition-colors {}", active_class)
            on:click=handle_click
        >
            // Group color indicator
            <div 
                class=format!("w-8 h-8 rounded flex items-center justify-center text-white text-xs font-bold mr-3 flex-shrink-0 {}", bg_color)
            >
                {initials}
            </div>
            
            // Group name and content
            <div class="flex-1 min-w-0">
                <span class="text-sm text-text-primary dark:text-text-primary-dark truncate block">
                    {group_name}
                </span>
            </div>
            
            // Role badge on the right
            {if membership.is_admin() {
                view! {
                    <span class="ml-2 px-3 text-xs bg-brand-primary-light text-white  py-1 rounded-md">
                        "Admin"
                    </span>
                }.into_view()
            } else if membership.is_member() {
                view! {
                    <span class="ml-2 px-3 text-xs bg-brand-secondary-light text-white  py-1 rounded-md">
                        "Member"
                    </span>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
        </div>
    }
}

#[component]
pub fn CreateGroupButton(
    #[prop(into)] on_create_click: Callback<()>,
) -> impl IntoView {
    view! {
        <div 
            class="flex items-center p-2 rounded cursor-pointer transition-colors hover:bg-bg-main dark:hover:bg-bg-main-dark text-brand-primary-light"
            on:click=move |_| on_create_click.call(())
        >
            // Plus icon
            <div class="w-8 h-8 rounded flex items-center justify-center border-2 border-dashed border-brand-primary-light mr-3 flex-shrink-0">
                <span class="text-sm font-bold">"+"</span>
            </div>
            
            // Create group text
            <span class="text-sm font-medium">
                "Crea nuovo gruppo"
            </span>
        </div>
    }
}

#[component]
pub fn ShowInvitesButton(
    #[prop(into)] on_show_invites_click: Callback<()>,
    #[prop(into, default = 0)] pending_count: usize,
) -> impl IntoView {
    view! {
        <div
            class="flex items-center p-2 rounded cursor-pointer transition-colors hover:bg-bg-main dark:hover:bg-bg-main-dark text-brand-primary-light"
            on:click=move |_| on_show_invites_click.call(())
        >
            <div class="w-8 h-8 rounded flex items-center justify-center mr-3 flex-shrink-0">
                <span class="text-sm font-bold">{"📮"}</span>
            </div>
            <span class="text-sm font-medium flex-1 text-left">
                "Inviti Ricevuti"
            </span>
            {if pending_count > 0 {
                view! {
                    <span class="ml-2 bg-red-500 text-white text-xs font-bold px-2 py-0.5 rounded-full">{pending_count}</span>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
        </div>
    }
}

fn get_group_initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    
    if words.len() >= 2 {
        // Se ci sono 2+ parole, usa la prima lettera di ciascuna delle prime due
        let first = words[0].chars().next().unwrap_or('?').to_uppercase().to_string();
        let second = words[1].chars().next().unwrap_or('?').to_uppercase().to_string();
        format!("{}{}", first, second)
    } else if let Some(word) = words.first() {
        // Se c'è una sola parola, usa le prime 2 lettere
        let chars: Vec<char> = word.chars().take(2).collect();
        if chars.len() >= 2 {
            format!("{}{}", chars[0].to_uppercase(), chars[1].to_uppercase())
        } else if chars.len() == 1 {
            format!("{}?", chars[0].to_uppercase())
        } else {
            "??".to_string()
        }
    } else {
        "??".to_string()
    }
}

fn generate_group_color(group_name: &str) -> String {
    // Mappa degli indici ai colori Tailwind per i gruppi
    let color_classes = [
        "bg-group-1",   // Viola principale
        "bg-group-2",   // Blu
        "bg-group-3",   // Verde
        "bg-group-4",   // Rosso
        "bg-group-5",   // Arancione
        "bg-group-6",   // Viola scuro
        "bg-group-7",   // Cyan
        "bg-group-8",   // Pink
        "bg-group-9",   // Lime
        "bg-group-10",  // Marrone
        "bg-group-11",  // Grigio scuro
        "bg-group-12",  // Blu scuro
    ];
    
    // Genera un hash dal nome del gruppo
    let mut hasher = DefaultHasher::new();
    group_name.to_lowercase().hash(&mut hasher);
    let hash = hasher.finish();
    
    // Seleziona una classe colore basata sull'hash
    let color_index = (hash % color_classes.len() as u64) as usize;
    color_classes[color_index].to_string()
}
