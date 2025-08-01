//! Build script per generare automaticamente i tipi dal server
//! 
//! Questo script viene eseguito durante `cargo build` e scarica
//! automaticamente i tipi dal server se disponibile

use std::process::Command;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Solo in development, non in release
    if env::var("PROFILE").unwrap_or_default() != "release" {
        generate_types_if_server_available();
    }
}

fn generate_types_if_server_available() {
    let server_url = env::var("RUGGINE_SERVER_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    // Testa se il server è raggiungibile
    let status = Command::new("powershell")
        .args(&[
            "-Command",
            &format!(
                "try {{ Invoke-RestMethod -Uri {}/api-docs/openapi.json -Method Get -TimeoutSec 2 | Out-Null; exit 0 }} catch {{ exit 1 }}",
                server_url
            )
        ])
        .status();

    if let Ok(exit_status) = status {
        if exit_status.success() {
            // Server disponibile - genera tipi aggiornati
            let _ = Command::new("powershell")
                .args(&["-ExecutionPolicy", "Bypass", "-File", "./generate-simple-types.ps1"])
                .status();
        }
        // Se il server non è disponibile, continua silenziosamente con i tipi esistenti
    }
}
