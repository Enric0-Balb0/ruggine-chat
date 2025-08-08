// Router modules
mod guards;
mod app_router;

// Re-export public components
pub use guards::AuthGuard;
pub use app_router::AppRouter;
