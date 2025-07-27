# Ruggine Client

Client application for the Ruggine chat system built with Leptos + Tauri.

## Architecture

This client is built using:
- **Leptos**: Modern Rust web framework for the UI
- **Tauri**: Cross-platform desktop app framework
- **WebAssembly (WASM)**: For high-performance frontend code

## Development

### Prerequisites

Make sure you have the following installed:
- Rust (latest stable)
- `trunk` CLI tool: `cargo install trunk`
- `tauri-cli`: `cargo install tauri-cli`

### Running in Development Mode

```bash
# Run the application in development mode
cargo tauri dev

# Or use the VS Code task: "Tauri: Dev Client"
```

### Building for Production

```bash
# Build the application for production
cargo tauri build

# Or use the VS Code task: "Tauri: Build Client"
```

### Frontend Only Development

If you want to develop only the frontend (Leptos) part:

```bash
# Serve the frontend with hot reload
trunk serve --open

# Or use the VS Code task: "Trunk: Serve Frontend"
```

## Project Structure

```
ruggine_client/
├── src/                    # Frontend Rust code (Leptos)
│   ├── main.rs            # Entry point
│   └── app.rs             # Main app component
├── src-tauri/             # Tauri backend
│   ├── src/
│   │   ├── main.rs        # Tauri main process
│   │   └── lib.rs         # Tauri commands
│   ├── Cargo.toml         # Tauri dependencies
│   └── tauri.conf.json    # Tauri configuration
├── public/                # Static assets
├── styles.css            # Global styles
├── index.html            # HTML template
├── Trunk.toml            # Trunk configuration
└── Cargo.toml            # Frontend dependencies
```

## Features to Implement

Based on the requirements document, this client will implement:

### User Management (FR1, FR7)
- [ ] User registration on first launch
- [ ] User authentication with unique ID
- [ ] User profile retrieval

### Group Chat Management (FR3)
- [ ] Create new group chats
- [ ] Send and accept group invitations
- [ ] View group participants and information

### Messaging (FR2)
- [ ] Send text messages to groups
- [ ] Receive real-time messages
- [ ] Display chat history

### Cross-Platform Support (FR4)
- [ ] Windows desktop support
- [ ] Linux desktop support  
- [ ] macOS desktop support
- [ ] Android mobile support (future)
- [ ] iOS mobile support (future)

### Performance Monitoring (FR5)
- [ ] CPU usage monitoring and logging
- [ ] Performance metrics display

## API Integration

The client will communicate with the Ruggine server (located in `../ruggine_server`) via:
- REST API for user management and group operations
- WebSocket for real-time messaging

## Configuration

- Development server runs on `http://localhost:1420`
- Tauri app configuration in `src-tauri/tauri.conf.json`
- Frontend build configuration in `Trunk.toml` + Leptos

This template should help get you started developing with Tauri and Leptos.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
