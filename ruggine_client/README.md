# Ruggine Client

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Leptos](https://img.shields.io/badge/leptos-0.6-blue.svg)](https://leptos.dev)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Enterprise-grade frontend client for the Ruggine chat system, built with modern Rust web technologies.

## 🏗️ Architecture Overview

### Technology Stack
- **Frontend Framework**: [Leptos 0.6](https://leptos.dev) - Type-safe reactive web framework
- **Styling**: [Tailwind CSS 3.4](https://tailwindcss.com) - Utility-first CSS framework
- **HTTP Client**: [Reqwest 0.12](https://github.com/seanmonstar/reqwest) - Async HTTP client with WASM support
- **Serialization**: [Serde](https://serde.rs) - High-performance serialization framework with JSON support
- **Date/Time**: [Chrono 0.4](https://github.com/chronotope/chrono) - Date and time library with WASM support
- **Storage**: [Gloo Storage](https://github.com/rustwasm/gloo) - Browser storage abstraction
- **Error Handling**: [ThisError](https://github.com/dtolnay/thiserror) - Derive macro for error types
- **Build Tool**: [Trunk](https://trunkrs.dev) - WASM web application bundler
- **Desktop Runtime**: [Tauri](https://tauri.app) - Cross-platform desktop application framework

## 🎨 Styling with Tailwind CSS

### 🚀 Recommended Development Workflow
**One-command development setup:**
```bash
# VS Code Command Palette (Ctrl+Shift+P)
Tasks: Run Task → Dev: Start All Frontend
```
This automatically starts both:
- **Trunk**: Frontend compilation and hot reload
- **Tailwind CSS Watcher**: Automatic CSS compilation

### Alternative Development Workflows
For manual control or advanced workflows:
```bash
# Start the Tailwind CSS watcher (runs in background)
npm run watch-css

# Or using PowerShell script
.\build-css.ps1 -Watch
```

### Manual CSS Build
For one-time CSS compilation:
```bash
# Build CSS once
npm run build-css

# Or using PowerShell script
.\build-css.ps1
```

### Available VS Code Tasks
Use the VS Code Command Palette (`Ctrl+Shift+P`):
- **🚀 Ruggine: Start Development** - **RECOMMENDED**: Start complete development environment
- `Tasks: Run Task` → `Trunk: Serve Frontend` - Frontend only
- `Tasks: Run Task` → `Tailwind: Watch CSS` - CSS watcher only  
- `Tasks: Run Task` → `Tailwind: Build CSS` - Build CSS once
- `Tasks: Run Task` → `Tauri: Dev Client` - Desktop app development

### Architectural Layers

```
┌─────────────────────────────────────────┐
│                UI Layer                 │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │  Components │  │     Pages       │   │
│  └─────────────┘  └─────────────────┘   │
├─────────────────────────────────────────┤
│               API Layer                 │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │API Services │  │   API Facade    │   │
│  │(Auth, User) │  │ (Unified Access)│   │
│  └─────────────┘  └─────────────────┘   │
│  ┌──────────────────────────────────┐   │
│  │         API Client (HTTP)        │   │
│  └──────────────────────────────────┘   │
├─────────────────────────────────────────┤
│             Utilities Layer             │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │   Storage   │  │     Theme       │   │
│  └─────────────┘  └─────────────────┘   │
├─────────────────────────────────────────┤
│              Type Layer                 │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │Server Types │  │  Common Types   │   │
│  │(Auth,User,.)│  │  (LoadingState) │   │
│  └─────────────┘  └─────────────────┘   │
└─────────────────────────────────────────┘
```

### Design Principles

- **Type Safety**: Leveraging Rust's type system for compile-time guarantees
- **Reactive Architecture**: Leptos signals for efficient UI updates  
- **Clear Separation**: API layer handles server communication, utils handle client-side concerns
- **Unified Access**: API facade provides consistent interface across all services
- **Testability**: Comprehensive unit and integration testing with 68+ test cases
- **Performance**: WASM compilation for near-native performance
- **Scalability**: Modular architecture supporting large-scale development

## 🚀 Quick Start

### Prerequisites

Ensure you have the following tools installed:

```bash
# Rust toolchain (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WASM target
rustup target add wasm32-unknown-unknown

# Development tools
cargo install trunk
cargo install tauri-cli
```

### Development Server

**🚀 RECOMMENDED - One-Command Setup:**
```bash
# Clone the repository
git clone <repository-url>
cd ruggine_client

# Start complete development environment (VS Code)
# Ctrl+Shift+P → Tasks: Run Task → 🚀 Ruggine: Start Development
```

**Alternative - Manual Setup:**
```bash
# Install dependencies and start development server
trunk serve --open

# In another terminal: start CSS watcher
npm run watch-css
```

The application will be available at `http://localhost:1420`

### Desktop Application

```bash
# Build and run desktop version
cargo tauri dev
```

## 📁 Project Structure

```
src/
├── api/                    # API communication layer
│   ├── services/          # API service implementations
│   │   ├── auth.rs        # Authentication API calls
│   │   ├── user.rs        # User management API calls
│   │   └── mod.rs
│   ├── client.rs          # HTTP client with auth support
│   ├── error.rs           # API error handling
│   ├── facade.rs          # Unified API facade
│   └── mod.rs
├── components/            # Reusable UI components
│   ├── app_layout.rs      # Main application layout
│   ├── app_navbar.rs      # Navigation bar component
│   ├── sidebar.rs         # Sidebar component
│   ├── theme_toggle.rs    # Theme switching component
│   └── mod.rs
├── config/                # Configuration and constants
│   ├── constants.rs       # Application constants (Auth, UI, Chat, etc.)
│   ├── endpoints.rs       # API endpoint definitions
│   ├── storage.rs         # Storage configuration and keys
│   └── mod.rs
├── error.rs               # Application error type definitions
├── hooks/                 # Leptos custom hooks
│   └── mod.rs
├── pages/                 # Application pages/views
│   ├── home.rs            # Home/dashboard page
│   ├── landing.rs         # Landing page
│   ├── login.rs           # Login page
│   ├── register.rs        # Registration page
│   └── mod.rs
├── router/                # Routing and navigation guards
│   ├── app_router.rs      # Main application router
│   ├── guards.rs          # Route guards (auth protection)
│   ├── login_guard.rs     # Login-specific guards
│   └── mod.rs
├── types/                 # Server-synchronized type definitions
│   ├── auth.rs            # Authentication types (LoginRequest, TokenResponse)
│   ├── common.rs          # Common utility types (ApiResponse, LoadingState)
│   ├── group.rs           # Group chat types (GroupChat, GroupChatCreateRequest)
│   ├── invitation.rs      # Invitation types (Invitation, InvitationStatus)
│   ├── user.rs            # User-related types (UserProfile, UserRegisterRequest)
│   └── mod.rs
├── utils/                 # Client-side utilities and services
│   ├── storage.rs         # Browser storage service (localStorage)
│   ├── theme.rs           # Theme management service
│   └── mod.rs
├── app.rs                 # Root application component
├── lib.rs                 # Library entry point
└── main.rs                # Application entry point

tests/
├── common/                # Test utilities and factories
│   ├── factory.rs         # Test data factories
│   └── mod.rs
├── integration/           # Integration tests
│   └── api/               # API client tests
├── unit/                  # Unit tests organized by module
│   └── types_test/        # Type-specific unit tests
├── e2e/                   # End-to-end tests (browser automation)
└── lib.rs                 # Test entry point

src-tauri/                 # Tauri desktop application backend
├── src/
│   ├── main.rs            # Tauri main process
│   └── lib.rs             # Tauri commands
├── Cargo.toml             # Tauri dependencies
└── tauri.conf.json        # Tauri configuration
```

## 🧪 Testing Strategy

### Test Types

1. **Unit Tests**: Individual component and service testing
2. **Integration Tests**: API client and service interaction testing
3. **E2E Tests**: Full user flow testing with browser automation

### Running Tests

```bash
# Run all tests (library + binary + integration)
cargo test

# Run unit tests only (embedded in type modules)
cargo test --lib

# Run integration tests only
cargo test --test lib

# Run with output for debugging
cargo test -- --nocapture

# Run specific test module
cargo test types::auth::tests
```

### Test Philosophy

- **Client-Focused Testing**: Tests focus on client-specific concerns, not server logic duplication
- **Comprehensive Coverage**: 68+ tests covering authentication, user management, and type validation
- **API Layer Testing**: Integration tests for API services and HTTP client
- **Client Services Testing**: Dedicated tests for storage and theme utilities
- **Embedded Unit Tests**: Type tests are embedded within type modules using `#[cfg(test)]`
- **Server-Synchronized Types**: All types in `src/types/` are synchronized with server OpenAPI definitions
- **Type Safety Validation**: Tests ensure proper serialization/deserialization and business logic methods

## 🔧 Development Guidelines

### Quick Start Development
1. **One Command**: `Ctrl+Shift+P` → `Tasks: Run Task` → **Dev: Start All Frontend**
2. **Code**: Modify Rust files and Tailwind classes
3. **Automatic**: Browser updates automatically with changes
4. **Optional**: Add `Tauri: Dev Client` for desktop app

### Code Organization

- **Layered Architecture**: Clean separation between API, UI, Utils, Router, and Types
- **API Layer**: Unified API communication with services (auth, user) and HTTP client
- **Client-side Services**: Storage and theme management in utils layer
- **Server-Synchronized Types**: Modern type system in `src/types/` with server OpenAPI synchronization
- **Configuration Management**: Centralized constants and endpoints in `src/config/`
- **Routing & Guards**: Comprehensive routing with authentication guards
- **Component System**: Reusable UI components with proper state management
- **Error Handling**: Comprehensive error handling with typed errors across all layers

### Coding Standards

```rust
// Example: API service method with proper error handling
use crate::api::services::auth::AuthService;
use crate::utils::storage::StorageService;
use crate::api::client::ApiClient;

impl AuthService {
    /// Login user with email and password
    /// 
    /// # Arguments
    /// * `email` - User email address (will be normalized)
    /// * `password` - User password (will be validated)
    /// 
    /// # Returns
    /// * `Ok(UserProfile)` - Successfully authenticated user
    /// * `Err(AuthError)` - Authentication failure details
    /// 
    /// # Example
    /// ```rust
    /// let api_client = ApiClient::new("http://localhost:8002");
    /// let storage = StorageService::new();
    /// let auth_service = AuthService::new(api_client, storage);
    /// let user = auth_service.login("test@example.com", "password").await?;
    /// ```
    pub async fn login(&self, email: String, password: String) -> Result<UserProfile, AuthError> {
        // Implementation...
    }
}
```

## 🔐 Security Considerations

- **Token Management**: Secure JWT token storage and automatic refresh
- **Input Validation**: Client-side validation with server-side verification
- **XSS Prevention**: Leptos provides built-in XSS protection
- **CSRF Protection**: Token-based authentication prevents CSRF attacks

## 📊 Performance Optimization

- **WASM Compilation**: Near-native performance in the browser
- **Code Splitting**: Lazy loading of components and routes
- **Reactive Updates**: Efficient DOM updates through Leptos signals
- **HTTP Caching**: Intelligent caching of API responses

## 🚀 Deployment

### Web Application
```bash
# Build for production
trunk build --release

# Serve static files from dist/
```

### Desktop Application
```bash
# Build desktop application
cargo tauri build
```

## 🔗 Integration with Ruggine Server

This client is designed to work with the [ruggine_server](../ruggine_server/README.md) backend:

- **API Compatibility**: Shared DTO definitions ensure type safety
- **Authentication**: JWT-based authentication with automatic token refresh
- **Error Handling**: Consistent error responses between client and server
- **Testing**: Coordinated testing strategies avoiding redundant endpoint testing

### Server Dependencies

- Server must be running on `http://localhost:8002` (configurable in `src/config/constants.rs`)
- Compatible with ruggine_server v1.0+
- Requires PostgreSQL database for user management
- 17 synchronized types maintained from server OpenAPI specifications

## 📚 API Documentation

### Authentication Flow

```rust
// Login example
use crate::api::services::auth::AuthService;
use crate::api::client::ApiClient;
use crate::utils::storage::StorageService;

let api_client = ApiClient::new("http://localhost:8002");
let storage = StorageService::new();
let auth_service = AuthService::new(api_client, storage);

let user_profile = auth_service.login("user@example.com", "password").await?;

// Check authentication status
if auth_service.is_authenticated() {
    let current_user = auth_service.get_current_user().unwrap();
}

// Logout
auth_service.logout().await?;
```

### Type Safety Example

```rust
// All API types are server-synchronized and type-safe
use crate::types::auth::{LoginRequest, TokenResponse};
use crate::types::user::{UserProfile, UserRegisterRequest, Gender, UserType};

let login_request = LoginRequest {
    email: "test@example.com".to_string(),
    password: "secure_password".to_string(),
};

// Type-safe business logic methods
let user = UserProfile { /* ... */ };
assert_eq!(user.full_name(), "John Doe");
assert!(user.is_active());
assert!(!user.is_admin());

// Server-synchronized enum variants
let gender = Gender::Male; // Serializes to "male"
let user_type = UserType::EndUser; // Serializes to "end_user"
```

## 🐛 Troubleshooting

### Common Issues

1. **WASM compilation errors**: Ensure `wasm32-unknown-unknown` target is installed
2. **Server connection issues**: Verify ruggine_server is running on port 8002 (configurable)
3. **Authentication failures**: Check server database and user credentials
4. **Type synchronization**: Ensure types match server OpenAPI definitions

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug trunk serve
```

## 🤝 Contributing

1. Follow Rust coding conventions
2. Maintain 100% type safety
3. Add tests for new functionality
4. Update documentation for API changes
5. Follow the established architectural patterns

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Built with ❤️ in Rust**
