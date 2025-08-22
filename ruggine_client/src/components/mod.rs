pub mod ui;

// Layout Components  
pub mod layout;

// Modal Components
pub mod modals;

// Chat Components
pub mod chat;

// Group Components
pub mod groups;

// Theme Components
pub mod theme;

// Re-exports for backward compatibility and ease of use
pub use ui::{
    UserAvatar,
    LucideIcon, IconSize,
    CircleLoader, InlineSpinner, LoadingButton,
    Toast, ToastMessage, ToastType, ToastProvider, ToastContainer, use_toast,
};

pub use layout::{AppLayout, AppNavbar, Sidebar};
pub use modals::{CreateGroupModal, InviteMemberModal, InviteMemberRequest, MemberRole, ViewMembersModal, GroupMember};
pub use chat::{ChatView, ChatHeaderAction, MessageInputArea};
pub use groups::{GroupItem, CreateGroupButton, ShowInvitesButton};
pub use theme::{ThemeToggle, ThemeSlider};
