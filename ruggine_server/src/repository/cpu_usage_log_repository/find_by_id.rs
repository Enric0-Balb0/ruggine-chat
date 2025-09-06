use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::cpu_usage_log::CpuUsageLog;
use crate::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;

impl CpuUsageLogRepository {
    pub async fn find_inner(&self, id: i32) -> Result<CpuUsageLog, Error> {
        let query = sqlx::query_as::<_, CpuUsageLog>(
            r#"
            SELECT id, timestamp, cpu_usage_percent
            FROM cpu_usage_log
            WHERE id = $1
            "#
        )
        .bind(id);

        // se c'è una transazione, usala; altrimenti usa la pool
        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            query.fetch_one(&mut *tx_ref).await
        } else {
            query.fetch_one(self.db_conn.get_pool()).await
        }
    }
}
