// Router modules
mod guards;
mod login_guard;
mod app_router;

// Re-export public components
pub use app_router::AppRouter;
pub use guards::AuthGuard;
