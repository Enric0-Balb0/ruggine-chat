use leptos::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[component]
pub fn UserAvatar(
    #[prop(into)] name: String,
    #[prop(into)] surname: String,
    #[prop(into)] username: String,
    #[prop(into, default = "md".to_string())] size: String,
) -> impl IntoView {
    // Genera le iniziali da nome e cognome
    let initials = get_initials(&name, &surname);
    
    // Genera un colore basato sullo username
    let bg_color = generate_avatar_color(&username);
    
    // Determina le dimensioni in base alla prop size
    let (width_class, height_class, text_class) = match size.as_str() {
        "sm" => ("w-6", "h-6", "text-xs"),
        "md" => ("w-8", "h-8", "text-xs"),
        "lg" => ("w-10", "h-10", "text-sm"),
        "xl" => ("w-12", "h-12", "text-base"),
        _ => ("w-8", "h-8", "text-xs"), // default
    };

    view! {
        <div 
            class=format!(
                "{} {} rounded-full flex items-center justify-center text-white font-bold cursor-pointer {} {}", 
                width_class, 
                height_class,
                text_class,
                bg_color
            )
        >
            {initials}
        </div>
    }
}

fn get_initials(name: &str, surname: &str) -> String {
    let name_initial = name.chars().next().unwrap_or('?').to_uppercase().to_string();
    let surname_initial = surname.chars().next().unwrap_or('?').to_uppercase().to_string();
    format!("{}{}", name_initial, surname_initial)
}

fn generate_avatar_color(username: &str) -> String {
    // Mappa degli indici ai colori Tailwind
    let color_classes = [
        "bg-avatar-1",   // Viola principale
        "bg-avatar-2",   // Verde
        "bg-avatar-3",   // Rosso
        "bg-avatar-4",   // Azzurro
        "bg-avatar-5",   // Viola chiaro
        "bg-avatar-6",   // Arancione
        "bg-avatar-7",   // Teal
        "bg-avatar-8",   // Lavanda
        "bg-avatar-9",   // Verde scuro
        "bg-avatar-10",  // Magenta
        "bg-avatar-11",  // Arancione scuro
        "bg-avatar-12",  // Blu
    ];
    
    // Genera un hash dallo username
    let mut hasher = DefaultHasher::new();
    username.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Seleziona una classe colore basata sull'hash
    let color_index = (hash % color_classes.len() as u64) as usize;
    color_classes[color_index].to_string()
}
