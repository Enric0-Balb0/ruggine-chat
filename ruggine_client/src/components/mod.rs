// UI Components
pub mod app_navbar; 
pub mod app_layout;
pub mod user_avatar;
pub mod sidebar;
pub mod group_item;
pub mod theme_toggle;

// Re-export components
pub use app_navbar::AppNavbar;
pub use app_layout::AppLayout;
pub use user_avatar::UserAvatar;
pub use sidebar::Sidebar;
pub use group_item::{GroupItem, CreateGroupButton};
pub use theme_toggle::ThemeToggle;

