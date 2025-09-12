# Ruggine Client

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Leptos](https://img.shields.io/badge/leptos-0.6-blue.svg)](https://leptos.dev)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Modern, type-safe chat application client built with Rust + Leptos + Tauri. Features real-time messaging, group management, and comprehensive WebSocket integration.

## 🚀 Quick Start

### Prerequisites
```bash
# Rust toolchain (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WASM target
rustup target add wasm32-unknown-unknown

# Development tools
cargo install trunk
cargo install tauri-cli
```

### One-Command Development Setup
**Recommended approach:**
```bash
# In VS Code: Ctrl+Shift+P → Tasks: Run Task → "Trunk: Serve Frontend"
# Or in terminal:
trunk serve --open
```

**With CSS watching (separate terminal):**
```bash
# VS Code: Ctrl+Shift+P → Tasks: Run Task → "Tailwind: Watch CSS"
# Or in terminal:
npm run watch-css
```

**Desktop application:**
```bash
cargo tauri dev
```

## 🏗️ Architecture Overview

### Technology Stack
- **Frontend Framework**: [Leptos 0.6](https://leptos.dev) with Client-Side Rendering
- **Styling**: [TailwindCSS 3.4](https://tailwindcss.com) with forms and typography plugins
- **HTTP Client**: [reqwest 0.12](https://github.com/seanmonstar/reqwest) with WASM support
- **WebSocket**: Real-time communication for chat and notifications
- **Storage**: [gloo-storage](https://github.com/rustwasm/gloo) for browser persistence
- **Build Tool**: [Trunk](https://trunkrs.dev) for WASM bundling
- **Desktop Runtime**: [Tauri](https://tauri.app) for cross-platform desktop apps
- **Testing**: Comprehensive suite with 229 tests (unit/integration/e2e)

### Application Architecture

```
┌─────────────────────────────────────────────────┐
│                  UI Layer                       │
│  ┌──────────────────┐  ┌────────────────────┐   │
│  │    Components    │  │       Pages        │   │
│  │  (chat, groups,  │  │ (home, landing,    │   │
│  │   layout, ui)    │  │  profile, etc.)    │   │
│  └──────────────────┘  └────────────────────┘   │
├─────────────────────────────────────────────────┤
│                Context Layer                    │
│  ┌──────────────────┐  ┌────────────────────┐   │
│  │   Auth Context   │  │ WebSocket Context  │   │
│  │  (user session)  │  │ (real-time comm)   │   │
│  └──────────────────┘  └────────────────────┘   │
├─────────────────────────────────────────────────┤
│                 API Layer                       │
│  ┌──────────────────┐  ┌────────────────────┐   │
│  │   API Services   │  │    API Facade      │   │
│  │ (auth, user,     │  │  (unified access)  │   │
│  │  group, message) │  │                    │   │
│  └──────────────────┘  └────────────────────┘   │
│  ┌─────────────────────────────────────────────┐│
│  │         HTTP Client + WebSocket             ││
│  └─────────────────────────────────────────────┘│
├─────────────────────────────────────────────────┤
│               Utilities Layer                   │
│  ┌──────────────────┐  ┌────────────────────┐   │
│  │     Storage      │  │       Theme        │   │
│  │   (local data)   │  │   (dark/light)     │   │
│  └──────────────────┘  └────────────────────┘   │
└─────────────────────────────────────────────────┘
```

### Core Features
- **Real-time Chat**: WebSocket-based messaging with typing indicators
- **Group Management**: Create, join, and manage chat groups
- **User Authentication**: JWT-based auth with automatic token refresh
- **Responsive Design**: Mobile-first design with TailwindCSS
- **Dark/Light Theme**: System and manual theme switching
- **Desktop Support**: Native desktop app via Tauri
- **Type Safety**: Server-synchronized types with 98 source files

## 📁 Project Structure

**Source Code (98 files):**
```
src/
├── api/                    # API communication (5 files)
│   ├── services/          # Service implementations
│   ├── ws/                # WebSocket handlers
│   ├── client.rs          # HTTP client with auth
│   ├── facade.rs          # Unified API access
│   └── http_error.rs      # Error handling
│
├── components/            # UI components (38 files)
│   ├── chat/              # Chat-specific components
│   ├── groups/            # Group management UI
│   ├── layout/            # App layout components
│   ├── modals/            # Modal dialogs
│   ├── theme/             # Theme components
│   └── ui/                # Reusable UI elements
│
├── pages/                 # Application pages (6 files)
│   ├── home.rs            # Main dashboard
│   ├── landing.rs         # Landing page
│   ├── profile.rs         # User profile
│   ├── register.rs        # User registration
│   └── cpu_logs.rs        # System monitoring
│
├── hooks/                 # Leptos custom hooks (11 files)
│   ├── use_groups.rs      # Group state management
│   ├── use_group_*.rs     # Chat-specific hooks
│   └── groups_provider.rs # Group data provider
│
├── context/               # Application contexts (4 files)
│   ├── auth_context.rs    # Authentication state
│   ├── unread_counts_context.rs # Notification counts
│   └── group_ws_contexts.rs # WebSocket contexts
│
├── types/                 # Type definitions (11 files)
│   ├── auth.rs            # Authentication types
│   ├── user.rs            # User profile types
│   ├── group.rs           # Group chat types
│   ├── message.rs         # Message types
│   ├── message_ws.rs      # WebSocket message types
│   └── invitation.rs      # Invitation types
│
├── config/                # Configuration (4 files)
│   ├── constants.rs       # App constants
│   ├── endpoints.rs       # API endpoints
│   └── storage.rs         # Storage configuration
│
├── utils/                 # Utilities (5 files)
│   ├── storage.rs         # Browser storage
│   ├── theme.rs           # Theme management
│   ├── timers.rs          # Timer utilities
│   └── error_messages.rs  # Error message handling
│
├── router/                # Routing system (3 files)
│   ├── app_router.rs      # Main router
│   ├── guards.rs          # Route guards
│   └── login_guard.rs     # Auth guards
│
├── assets/                # Static assets
├── styles/                # TailwindCSS files
├── app.rs                 # Root component
├── main.rs                # Application entry point
└── lib.rs                 # Library definitions
```

**Test Suite (57 files, 229 tests):**
```
tests/
├── common/                # Test utilities (7 files)
│   ├── factories/         # Modular test factories
│   │   ├── base_factory.rs
│   │   ├── auth_factory.rs
│   │   ├── user_factory.rs
│   │   ├── group_factory.rs
│   │   ├── message_factory.rs
│   │   ├── websocket_factory.rs
│   │   └── ui_factory.rs
│   └── mod.rs
│
├── integration/           # Integration tests (46 tests)
│   ├── api/               # API service tests
│   └── services/          # Service integration tests
│
├── unit/                  # Unit tests (organized by module)
│   ├── services/          # Service unit tests
│   ├── components/        # Component tests
│   └── utils/             # Utility tests
│
├── e2e/                   # End-to-end tests
└── lib.rs                 # Test entry point
```

**Desktop Integration:**
```
src-tauri/                 # Tauri desktop backend
├── src/
│   ├── main.rs            # Tauri main process
│   └── lib.rs             # Tauri commands
├── Cargo.toml             # Tauri dependencies
└── tauri.conf.json        # Desktop app configuration
```

## 🧪 Testing Strategy

### Test Coverage (229 total tests)
- **Integration Tests**: 46 tests covering API services and WebSocket communication
- **Unit Tests**: 153 tests for components, utilities, and business logic
- **E2E Tests**: 30 tests for complete user workflows
- **Test Status**: ✅ All tests passing (225 passed, 0 failed, 4 ignored)

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test categories
cargo test --test integration_tests
cargo test --lib  # Unit tests only

# Run with debug output
cargo test -- --nocapture

# Run specific test
cargo test auth_service_login
```

### Test Organization
- **Modular Factories**: 7 specialized test factories for type-safe test data
- **Client-Focused**: Tests focus on client-specific logic, not server duplication
- **Comprehensive Coverage**: Authentication, user management, chat functionality, WebSocket communication

## 🔧 Development Workflow

### Available VS Code Tasks
- **Trunk: Serve Frontend** - Start development server with hot reload
- **Tailwind: Watch CSS** - Automatic CSS compilation
- **Tailwind: Build CSS** - One-time CSS build
- **Tauri: Dev Client** - Desktop development mode
- **Generate API Types** - Sync types with server OpenAPI spec

### Development Guidelines

**Type Safety:**
```rust
// All types are server-synchronized
use crate::types::{auth::LoginRequest, user::UserProfile};

// Business logic methods on types
let user = UserProfile { /* ... */ };
assert_eq!(user.full_name(), "John Doe");
assert!(user.is_active());
```

**API Communication:**
```rust
// Unified API facade pattern
use crate::api::facade::ApiFacade;

let api = ApiFacade::new();
let user = api.auth().login(email, password).await?;
let groups = api.groups().get_user_groups().await?;
```

**WebSocket Integration:**
```rust
// Real-time communication hooks
use crate::hooks::{use_group_message_ws, use_app_group_ws};

let message_ws = use_group_message_ws(group_id);
let group_ws = use_app_group_ws();
```

## 🎨 Styling and UI

### TailwindCSS Integration
- **Utility-First**: Comprehensive TailwindCSS classes
- **Responsive Design**: Mobile-first approach
- **Dark/Light Theme**: System preference detection
- **Component System**: Reusable UI components

### Theme Management
```rust
// Theme context and utilities
use crate::utils::theme::{ThemeProvider, use_theme};

let theme = use_theme();
theme.toggle(); // Switch between light/dark
```

## 🔌 Real-time Features

### WebSocket Communication
- **Message Streaming**: Real-time message delivery
- **Typing Indicators**: Live typing status
- **Group Notifications**: Member join/leave events
- **Unread Counts**: Real-time notification badges

### Hook System
- `use_group_message_ws`: Message WebSocket management
- `use_app_group_ws`: Global group WebSocket
- `use_groups`: Group state management
- `use_group_user_cache`: User data caching

## 🚀 Build and Deployment

### Web Application
```bash
# Development
trunk serve --open

# Production build
trunk build --release
# Outputs to dist/
```

### Desktop Application
```bash
# Development
cargo tauri dev

# Production build
cargo tauri build
# Outputs platform-specific binaries
```

### CSS Pipeline
```bash
# Watch mode (development)
npm run watch-css

# Build mode (production)
npm run build-css
```

### Required Server Features
- ruggine_server v1.0+
- PostgreSQL database
- WebSocket support for real-time features
- JWT authentication endpoints

## 🐛 Troubleshooting

### Common Issues
1. **WASM Target Missing**: `rustup target add wasm32-unknown-unknown`
2. **Server Connection**: Verify ruggine_server is running on port 8002
3. **CSS Not Loading**: Run `npm run build-css` or start CSS watcher
4. **Authentication Issues**: Check server database and JWT configuration

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=debug trunk serve

