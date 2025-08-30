pub mod profile;
pub use profile::ProfilePage;
// Pages module - UI pages/views
pub mod landing;
pub mod register;
pub mod home;
pub mod cpu_logs;

// Re-export pages
pub use landing::LandingPage;
pub use register::RegisterPage;
pub use home::HomePage;
pub use cpu_logs::CpuLogsPage;
