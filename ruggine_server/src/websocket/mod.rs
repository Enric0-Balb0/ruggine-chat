pub mod core;
pub mod message;

// Re-export delle strutture principali per facilità d'uso
pub use core::manager::{WebSocketManager, ConnectionStats};
pub use core::connection::{WebSocketConnection, handle_websocket_connection};
pub use message::{WebSocketMessage, GroupAction, InvitationAction, NotificationLevel};