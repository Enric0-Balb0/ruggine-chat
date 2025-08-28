use crate::dto::cpu_usage_log_dto::CpuUsageLogReadDto;
use crate::error::api_error::ApiError;
use crate::error::cpu_usage_log_error::CpuUsageLogError;
use crate::error::db_error::DbError;
use crate::service::cpu_usage_log_service::CpuUsageLogService;

impl CpuUsageLogService {
    pub async fn find_by_id_internal(&self, id: i32) -> Result<CpuUsageLogReadDto, ApiError> {
        let text_message = self.cpu_usage_log_repo.find(id).await.map_err(|e| {
            let db_error = match e {
                sqlx::Error::RowNotFound => {
                    return ApiError::CpuUsageLogError(CpuUsageLogError::CpuUsageLogNotFound);
                }
                _ => ApiError::DbError(DbError::SomethingWentWrong(e.to_string())),
            };
            db_error
        })?;

        Ok(CpuUsageLogReadDto::from(text_message))
    }
}

#[cfg(test)]
mod find_by_id_service_tests {
    use super::*;
    use crate::factory::cpu_usage_log_factory::CpuUsageLogFactory;
    use crate::repository::cpu_usage_log_repository::cpu_usage_log_repository_trait::MockCpuUsageLogRepositoryTrait;
    use crate::service::cpu_usage_log_service::CpuUsageLogService;
    use crate::service::group_membership_service::group_membership_service_trait::MockGroupMembershipServiceTrait;
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_find_by_id_success() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let cpu_usage_log = CpuUsageLogFactory::fake_cpu_usage_log();
        let expected_dto = CpuUsageLogReadDto::from(cpu_usage_log.clone());
        
        mock_repo
            .expect_find()
            .with(eq(1))
            .times(1)
            .returning({
                let cpu_usage_log = cpu_usage_log.clone();
                move |_| Box::pin({
                    let cpu_usage_log = cpu_usage_log.clone();
                    async move { Ok(cpu_usage_log) }
                })
            });

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.find_by_id_internal(1).await;

        // Assert
        assert!(result.is_ok());
        let actual_dto = result.unwrap();
        assert_eq!(actual_dto.id, expected_dto.id);
        assert_eq!(actual_dto.timestamp, expected_dto.timestamp);
        assert_eq!(actual_dto.cpu_usage_percent, expected_dto.cpu_usage_percent);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        
        mock_repo
            .expect_find()
            .with(eq(999))
            .times(1)
            .returning(|_| Box::pin(async move { Err(sqlx::Error::RowNotFound) }));

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.find_by_id_internal(999).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::CpuUsageLogError(CpuUsageLogError::CpuUsageLogNotFound) => {
                // Expected error type
            }
            _ => panic!("Expected CpuUsageLogNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_find_by_id_database_error() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let mock_group_member_membership_service = MockGroupMembershipServiceTrait::new();

        mock_repo
            .expect_find()
            .with(eq(1))
            .times(1)
            .returning(|_| Box::pin(async move { 
                Err(sqlx::Error::Configuration("Database connection failed".into())) 
            }));

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.find_by_id_internal(1).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::SomethingWentWrong(_)) => {
                // Expected error type
            }
            _ => panic!("Expected SomethingWentWrong error"),
        }
    }
}
