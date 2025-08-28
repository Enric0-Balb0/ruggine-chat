use crate::repository::cpu_usage_log_repository::cpu_usage_log_repository_trait::{CpuUsageLogRepositoryTrait};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(Clone)]
pub struct CpuUsageLogService {
    pub(crate) cpu_usage_log_repo: Arc<dyn CpuUsageLogRepositoryTrait>,
    pub(crate) handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    pub(crate) is_running: Arc<Mutex<bool>>,
    pub(crate) monitoring_interval_ms: Option<u64>,
}

impl CpuUsageLogService {
    pub fn new(
        cpu_usage_log_repo: Arc<dyn CpuUsageLogRepositoryTrait>
    ) -> Self {
        Self {
            cpu_usage_log_repo,
            handle: Arc::new(Mutex::new(None)),
            is_running: Arc::new(Mutex::new(false)),
            monitoring_interval_ms: None
        }
    }

    pub fn set_monitoring_interval_ms_internal(&mut self, interval_ms: u64) {
        self.monitoring_interval_ms = Some(interval_ms);
    }
    
    pub fn cpu_usage_log_repo(&self) -> Arc<dyn CpuUsageLogRepositoryTrait> {
        Arc::clone(&self.cpu_usage_log_repo)
    }
}
