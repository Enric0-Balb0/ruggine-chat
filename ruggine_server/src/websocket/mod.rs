pub mod core;
pub mod message;
pub mod group_message;
pub mod notification_message;

// Re-export delle strutture principali per facilità d'uso
pub use core::manager::{WebSocketManager};
pub use core::connection::{WebSocketConnection};
pub use message::*;
pub use notification_message::*;
pub use group_message::*;