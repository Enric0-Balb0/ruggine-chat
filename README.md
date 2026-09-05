# Ruggine Chat

## Overview
Ruggine Chat is a cross-platform client-server chat application developed in Rust. Designed for the Programmazione di Sistema course, the project emphasizes memory safety, multi-threading, and optimal resource utilization. 

## System Architecture
The application is structured into two main components:
- **`ruggine_server`**: The backend TCP server responsible for managing incoming connections, routing messages, handling user sessions, and enforcing business logic constraints (e.g., group invitations, authentication). It leverages thread-safe structures (`Arc`, `Mutex`) and asynchronous channels to support high concurrency.
- **`ruggine_client`**: The desktop GUI client that connects to the server and parses incoming data streams. 

## Technical Specifications
- **Concurrency & Networking**: Custom TCP-based communication protocol designed to handle multiple simultaneous client connections without blocking.
- **Resource Optimization**: Engineered for minimal system footprint, strictly adhering to an executable size limit of under 30MB.
- **Performance Telemetry**: Includes an automated background telemetry system that records and logs the server's CPU usage to a database every two minutes for performance tracking.
- **Cross-Platform Support**: Tested and compiled for execution across Windows, Linux, macOS, and Android platforms.

## Build and Execution

### Requirements
- Rust Toolchain (1.70+)

### Instructions
1. **Server Initialization**:
```bash
cd ruggine_server
cargo run --release
```

2. **Client Initialization**:
```bash
cd ruggine_client
cargo run --release
```
