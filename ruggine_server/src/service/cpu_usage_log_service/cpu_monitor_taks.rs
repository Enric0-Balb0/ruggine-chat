use std::sync::Arc;
use std::time::Duration;
use bigdecimal::{BigDecimal, FromPrimitive};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::interval;
use sysinfo::System;
use tracing::{error, info};
use crate::dto::cpu_usage_log_dto::CpuUsageLogCreateDto;
use crate::entity::cpu_usage_log::CpuUsageLog;
use crate::error::api_error::ApiError;
use crate::error::cpu_usage_log_error::CpuUsageLogError;
use crate::service::cpu_usage_log_service::{CpuUsageLogService, CpuUsageLogServiceTrait};

impl CpuUsageLogService{
    /// Starts a CPU monitoring task that logs CPU usage
    pub async fn start_monitoring_internal(self: Arc<Self>) -> Result<(), ApiError> {
        let mut is_running = self.is_running.lock().await;

        if self.monitoring_interval_ms.is_none() {
            return Err(ApiError::CpuUsageLogError(CpuUsageLogError::CpuMonitoringIntervalMustBeSet));
        }

        let monitoring_interval_ms = self.monitoring_interval_ms.unwrap();

        if *is_running {
            return Err(ApiError::CpuUsageLogError(CpuUsageLogError::CpuMonitoringAlreadyRunning));
        }

        *is_running = true;
        let self_clone = Arc::clone(&self);
        let is_running_clone = Arc::clone(&self.is_running);

        let handle: JoinHandle<()> = tokio::spawn(async move {
            let mut system = System::new_all();
            let mut interval = interval(Duration::from_millis(monitoring_interval_ms));

            info!("{}", format!("🖥️  CPU monitoring task started - reports every {} milliseconds", monitoring_interval_ms));

            loop {
                interval.tick().await;

                // Check if the task should stop
                if !*is_running_clone.lock().await {
                    info!("🛑 CPU monitoring task stopped");
                    break;
                }

                // Refresh system information
                system.refresh_cpu_all();

                // Calculate average CPU usage
                let cpu_usage = Self::calculate_average_cpu_usage(&system);

                // Log CPU usage
                info!("🔄 Server CPU usage: {:.2}%", cpu_usage);

                // Save to the database
                if let Some(cpu_usage_percent) = BigDecimal::from_f32(cpu_usage) {
                    match self_clone.create(CpuUsageLogCreateDto { cpu_usage_percent }).await {
                        Ok(cpu_usage_log) => {
                            info!("✅ CPU usage log saved with ID {}", cpu_usage_log.id);
                        },
                        Err(err) => {
                            error!("❌ Failed to save CPU usage log: {:?}", err);
                        }
                    }
                } else {
                    error!("❌ Failed to convert CPU usage from f32 to BigDecimal");
                }
            }
        });

        // Save the task handle in the service
        let mut handle_lock = self.handle.lock().await;
        *handle_lock = Some(handle);

        Ok(())
    }

    /// Stop monitoring CPI task
    pub async fn stop_monitoring_internal(&mut self) -> Result<(), ApiError> {
        let mut is_running = self.is_running.lock().await;
        
        if !*is_running {
            return Err(ApiError::CpuUsageLogError(CpuUsageLogError::CpuMonitoringNotRunning));
        }

        *is_running = false;

        let mut handle_lock = self.handle.lock().await;
        if let Some(handle) = handle_lock.take() {
            handle.abort();
            info!("🛑 Monitoring CPU task stopped successfully");
        }

        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }

    fn calculate_average_cpu_usage(system: &System) -> f32 {
        let cpus = system.cpus();
        if cpus.is_empty() {
            return 0.0;
        }

        let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
        total_usage / cpus.len() as f32
    }

    pub fn clone_for_arc(&self) -> Self {
        Self {
            cpu_usage_log_repo: Arc::clone(&self.cpu_usage_log_repo),
            handle: Arc::clone(&self.handle),
            is_running: Arc::clone(&self.is_running),
            monitoring_interval_ms: self.monitoring_interval_ms.clone()
        }
    }
}

