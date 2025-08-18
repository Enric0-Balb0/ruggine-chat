// UI Components
pub mod app_navbar; 
pub mod app_layout;
pub mod user_avatar;
pub mod sidebar;
pub mod group_item;
pub mod theme_toggle;
pub mod theme_slider;
pub mod create_group_modal;
pub mod circle_loader;
pub mod lucide_icon;
pub mod toast;
pub mod toast_provider;

// Re-export components
pub use app_navbar::AppNavbar;
pub use app_layout::AppLayout;
pub use user_avatar::UserAvatar;
pub use sidebar::Sidebar;
pub use group_item::{GroupItem, CreateGroupButton};
pub use theme_toggle::ThemeToggle;
pub use theme_slider::ThemeSlider;
pub use create_group_modal::CreateGroupModal;
pub use circle_loader::{CircleLoader, InlineSpinner, LoadingButton};
pub use lucide_icon::{LucideIcon, IconSize};
pub use toast::{Toast, ToastMessage, ToastType};
pub use toast_provider::{ToastProvider, ToastContainer, use_toast};

