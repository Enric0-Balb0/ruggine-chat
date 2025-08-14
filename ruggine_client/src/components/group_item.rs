use leptos::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[component]
pub fn GroupItem(
    #[prop(into)] name: String,
    #[prop(into, default = false)] is_active: bool,
) -> impl IntoView {
    // Genera un colore basato sul nome del gruppo
    let bg_color = generate_group_color(&name);
    
    // Genera le iniziali del gruppo (prime 2 lettere o prime lettere di 2 parole)
    let initials = get_group_initials(&name);

    let active_class = if is_active {
        "bg-bg-main dark:bg-bg-main-dark border-l-2 border-brand-primary-light"
    } else {
        "hover:bg-bg-main dark:hover:bg-bg-main-dark"
    };

    view! {
        <div class=format!("flex items-center p-2 rounded cursor-pointer transition-colors {}", active_class)>
            // Group color indicator
            <div 
                class=format!("w-8 h-8 rounded flex items-center justify-center text-white text-xs font-bold mr-3 flex-shrink-0 {}", bg_color)
            >
                {initials}
            </div>
            
            // Group name
            <span class="text-sm text-text-primary dark:text-text-primary-dark truncate flex-1">
                {name}
            </span>
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
