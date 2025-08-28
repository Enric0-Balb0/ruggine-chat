use crate::config::database::Database;
use crate::service::cpu_usage_log_service::CpuUsageLogServiceTrait;
use crate::utils::service_initializer::ServiceInitializer;
use std::sync::Arc;

#[derive(Clone)]
pub struct CpuUsageLogState {
    pub cpu_usage_log_service: Arc<dyn CpuUsageLogServiceTrait>,
}

impl CpuUsageLogState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        let service_init = ServiceInitializer::new(db_conn);
        let cpu_usage_log_service = service_init.cpu_usage_log_service();

        Self {
            cpu_usage_log_service,
        }
    }


    pub fn with_dependencies(
        cpu_usage_log_service: Arc<dyn CpuUsageLogServiceTrait>,
    ) -> Self {
        Self {
            cpu_usage_log_service,
        }
    }
}
