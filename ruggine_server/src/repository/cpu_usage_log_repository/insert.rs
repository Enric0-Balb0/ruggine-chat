use sqlx::Error;
use tracing::{error, info};
use crate::config::database::DatabaseTrait;
use crate::entity::cpu_usage_log::NewCpuUsageLog;
use crate::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;

impl CpuUsageLogRepository {
    pub async fn insert_inner(&self, new_cpu_usage_log: NewCpuUsageLog) -> Result<i32, Error> {
        let now = chrono::Utc::now();
        let query = sqlx::query_scalar(
            r#"
            INSERT INTO cpu_usage_log (cpu_usage_percent)
            VALUES ($1)
            RETURNING id
            "#
        )
        .bind(new_cpu_usage_log.cpu_usage_percent)
        .bind(now);

        // se c'è una transazione, usala; altrimenti usa la pool
        let response;

        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            info!("Using transaction");
            response = query.fetch_one(&mut *tx_ref).await
        } else {
            info!("Using pool");
            response = query.fetch_one(self.db_conn.get_pool()).await
        }

        match response {
            Ok(id) => Ok(id),
            Err(err) => {
                error!("Failed to insert cpu usage log repository: {:?}", err);
                Err(err)
            }
        }
    }
}
