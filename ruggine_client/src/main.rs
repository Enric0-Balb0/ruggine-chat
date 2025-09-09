mod app;
mod context;
mod types;
mod api;
mod hooks;
mod components;
mod pages;
mod utils;
mod error;
mod config;
mod router;

use app::*;
use leptos::*;

fn main() {
    console_error_panic_hook::set_once();
    
    console_log::init_with_level(log::Level::Info).expect("Errore nell'inizializzazione del logger");
        
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
