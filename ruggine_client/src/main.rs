mod app;
mod types;
mod http;
mod api;
mod services;
mod hooks;
mod components;
mod pages;
mod utils;
mod error;
mod dto;
mod config;

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
