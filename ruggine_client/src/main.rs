mod app;
mod types;
mod api;
mod hooks;
mod components;
mod pages;
mod utils;
mod error;
mod dto;
mod config;
mod router;

use app::*;
use leptos::*;

fn main() {
    console_error_panic_hook::set_once();
    
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
