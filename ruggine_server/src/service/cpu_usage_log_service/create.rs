use crate::config::database::DatabaseTrait;
use crate::dto::cpu_usage_log_dto::{CpuUsageLogCreateDto, CpuUsageLogReadDto};
use crate::entity::group_membership::MembershipStatus;
use crate::error::api_error::ApiError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::error::group_chat_error::GroupChatError;
use crate::error::cpu_usage_log_error::CpuUsageLogError;
use crate::error::db_error::DbError;
use crate::entity::cpu_usage_log::NewCpuUsageLog;
use crate::service::cpu_usage_log_service::{CpuUsageLogService, CpuUsageLogServiceTrait};

impl CpuUsageLogService {
    pub async fn create_internal(
        &self,
        payload: CpuUsageLogCreateDto,
    ) -> Result<CpuUsageLogReadDto, ApiError> {
        // Create the new text cpu_usage_log entity
        let new_cpu_usage_log = NewCpuUsageLog::from(payload);

        // Insert the cpu_usage_log into the database
        let cpu_usage_log_id = self
            .cpu_usage_log_repo
            .insert(new_cpu_usage_log)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(db_err) => {
                    if let Some(code) = db_err.code() {
                        if code == "23503" {
                            return ApiError::DbError(DbError::ForeignKeyViolation(db_err.to_string()));
                        }
                    }
                    ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string()))
                }
                _ => ApiError::DbError(DbError::SomethingWentWrong(e.to_string())),
            })?;

        let cpu_usage_log = self.find_by_id(cpu_usage_log_id).await?;

        Ok(cpu_usage_log)
    }
}

#[cfg(test)]
mod create_service_tests {
    use super::*;
    use crate::factory::cpu_usage_log_factory::CpuUsageLogFactory;
    use crate::repository::cpu_usage_log_repository::cpu_usage_log_repository_trait::MockCpuUsageLogRepositoryTrait;
    use crate::utils::mock_database_error::MockDatabaseError;
    use bigdecimal::BigDecimal;
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_cpu_usage_log_success() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let cpu_usage_log_id = 1;
        let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(75));
        let mut expected_cpu_usage_log = CpuUsageLogFactory::fake_cpu_usage_log_with_id(cpu_usage_log_id);
        expected_cpu_usage_log.cpu_usage_percent = payload.cpu_usage_percent.clone();
        let expected_dto = CpuUsageLogReadDto::from(expected_cpu_usage_log.clone());
        
        // Mock successful insert
        mock_repo
            .expect_insert()
            .with(function({
                let payload = payload.clone();
                move |new_log: &NewCpuUsageLog| {
                    new_log.cpu_usage_percent == payload.cpu_usage_percent
                }
            }))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(cpu_usage_log_id) }));
        
        // Mock successful find after insert (this simulates calling find_by_id)
        mock_repo
            .expect_find()
            .with(eq(cpu_usage_log_id))
            .times(1)
            .returning({
                let expected_cpu_usage_log = expected_cpu_usage_log.clone();
                move |_| Box::pin({
                    let expected_cpu_usage_log = expected_cpu_usage_log.clone();
                    async move { Ok(expected_cpu_usage_log) }
                })
            });

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.create_internal(payload.clone()).await;

        // Assert
        assert!(result.is_ok(), "Failed to create CPU usage log: {:?}", result.err());
        let created_log = result.unwrap();
        assert_eq!(created_log.id, cpu_usage_log_id);
        assert_eq!(created_log.cpu_usage_percent, payload.cpu_usage_percent);
    }

    #[tokio::test]
    async fn test_create_cpu_usage_log_insert_fails_foreign_key() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto();

        // Mock failed insert with foreign key violation
        mock_repo
            .expect_insert()
            .times(1)
            .returning(|_| Box::pin(async move { 
                Err(MockDatabaseError::foreign_key_violation()) 
            }));

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.create_internal(payload).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::DbError(DbError::ForeignKeyViolation(_))));
    }

    #[tokio::test]
    async fn test_create_cpu_usage_log_insert_fails_generic_error() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto();

        // Mock failed insert with generic database error
        mock_repo
            .expect_insert()
            .times(1)
            .returning(|_| Box::pin(async move { 
                Err(sqlx::Error::Configuration("Database connection failed".into())) 
            }));

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.create_internal(payload).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::DbError(DbError::SomethingWentWrong(_))));
    }

    #[tokio::test]
    async fn test_create_cpu_usage_log_find_after_insert_fails() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let cpu_usage_log_id = 1;
        let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto();

        // Mock successful insert
        mock_repo
            .expect_insert()
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(cpu_usage_log_id) }));
        
        // Mock failed find after insert
        mock_repo
            .expect_find()
            .with(eq(cpu_usage_log_id))
            .times(1)
            .returning(|_| Box::pin(async move { Err(sqlx::Error::RowNotFound) }));

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.create_internal(payload).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::CpuUsageLogError(CpuUsageLogError::CpuUsageLogNotFound)));
    }

    #[tokio::test]
    async fn test_create_cpu_usage_log_with_zero_percent() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let cpu_usage_log_id = 1;
        let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(0));
        let expected_cpu_usage_log = CpuUsageLogFactory::fake_cpu_usage_log_with_percent(BigDecimal::from(0));
        
        // Mock successful insert
        mock_repo
            .expect_insert()
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(cpu_usage_log_id) }));
        
        // Mock successful find after insert
        mock_repo
            .expect_find()
            .with(eq(cpu_usage_log_id))
            .times(1)
            .returning({
                let expected_cpu_usage_log = expected_cpu_usage_log.clone();
                move |_| Box::pin({
                    let expected_cpu_usage_log = expected_cpu_usage_log.clone();
                    async move { Ok(expected_cpu_usage_log) }
                })
            });

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.create_internal(payload.clone()).await;

        // Assert
        assert!(result.is_ok());
        let created_log = result.unwrap();
        assert_eq!(created_log.cpu_usage_percent, BigDecimal::from(0));
    }

    #[tokio::test]
    async fn test_create_cpu_usage_log_with_high_percent() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let cpu_usage_log_id = 1;
        let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(99));
        let expected_cpu_usage_log = CpuUsageLogFactory::fake_cpu_usage_log_with_percent(BigDecimal::from(99));
        
        // Mock successful insert
        mock_repo
            .expect_insert()
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(cpu_usage_log_id) }));
        
        // Mock successful find after insert
        mock_repo
            .expect_find()
            .with(eq(cpu_usage_log_id))
            .times(1)
            .returning({
                let expected_cpu_usage_log = expected_cpu_usage_log.clone();
                move |_| Box::pin({
                    let expected_cpu_usage_log = expected_cpu_usage_log.clone();
                    async move { Ok(expected_cpu_usage_log) }
                })
            });

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.create_internal(payload.clone()).await;

        // Assert
        assert!(result.is_ok());
        let created_log = result.unwrap();
        assert_eq!(created_log.cpu_usage_percent, BigDecimal::from(99));
    }
}
