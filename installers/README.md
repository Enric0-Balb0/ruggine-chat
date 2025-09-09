# Installers

Questa cartella contiene gli installer dell'applicazione Ruggine per tutti i sistemi operativi supportati.

## Struttura

```
installers/
├── windows/          # Installer per Windows (.msi, .exe)
├── macos/           # Installer per macOS (.dmg, .app)
├── linux/           # Installer per Linux (.deb, .rpm, .AppImage)
└── README.md        # Questo file
```

## Come Generare gli Installer

### Prerequisiti
- Rust e Cargo installati
- Tauri CLI installato: `cargo install tauri-cli`
- Per Windows: WiX Toolset (per .msi)
- Per macOS: Xcode Command Line Tools
- Per Linux: Dipendenze specifiche per il packaging

### Comandi di Build

#### Windows
```bash
cd ruggine_client
cargo tauri build --target x86_64-pc-windows-msvc
```

#### macOS
```bash
cd ruggine_client
cargo tauri build --target x86_64-apple-darwin
# o per Apple Silicon:
cargo tauri build --target aarch64-apple-darwin
```

#### Linux
```bash
cd ruggine_client
cargo tauri build --target x86_64-unknown-linux-gnu
```

## Copiare gli Installer

Dopo la build, copia gli installer dalla cartella `target/release/bundle/` alle rispettive cartelle:

### Windows
- File `.msi`: `target/release/bundle/msi/` → `installers/windows/`
- File `.exe`: `target/release/bundle/nsis/` → `installers/windows/`

### macOS
- File `.dmg`: `target/release/bundle/dmg/` → `installers/macos/`
- File `.app`: `target/release/bundle/macos/` → `installers/macos/`

### Linux
- File `.deb`: `target/release/bundle/deb/` → `installers/linux/`
- File `.rpm`: `target/release/bundle/rpm/` → `installers/linux/`
- File `.AppImage`: `target/release/bundle/appimage/` → `installers/linux/`

## Note

- Gli installer in questa cartella sono tracciati da git per facilitare la distribuzione
- Aggiorna sempre la versione in `Cargo.toml` prima di generare nuovi installer
- Testa sempre gli installer su macchine pulite prima della distribuzione

## Versioning

Usa il formato semantico per le versioni: `MAJOR.MINOR.PATCH`
- MAJOR: Cambiamenti incompatibili
- MINOR: Nuove funzionalità compatibili
- PATCH: Bug fix compatibili
