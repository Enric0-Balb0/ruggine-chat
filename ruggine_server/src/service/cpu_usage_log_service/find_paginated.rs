use crate::dto::cpu_usage_log_dto::{CpuUsageLogReadDto};
use crate::dto::cpu_usage_log_pagination_dto::CpuUsageLogPaginationQuery;
use crate::entity::group_membership::MembershipStatus;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_chat_error::GroupChatError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::error::cpu_usage_log_error::CpuUsageLogError;
use crate::response::paginated_response::PaginationMetadata;
use crate::response::PaginatedCpuUsageLogResponse;
use crate::service::cpu_usage_log_service::{CpuUsageLogService, CpuUsageLogServiceTrait};
use chrono::{DateTime, Utc};

impl CpuUsageLogService {
    pub async fn find_paginated_internal(
        &self,
        pagination_query: CpuUsageLogPaginationQuery,
    ) -> Result<PaginatedCpuUsageLogResponse, ApiError> {
        // Il cursor è già validato automaticamente durante la deserializzazione del DTO
        let limit = pagination_query.limit;
        let cursor_datetime = pagination_query.cursor;

        let cpu_usage_logs = self
            .cpu_usage_log_repo
            .find_paginated(cursor_datetime, limit + 1)
            .await
            .map_err(|e| {
                let db_error = match e {
                    sqlx::Error::Database(db_err) => {
                        ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string()))
                    }
                    _ => ApiError::DbError(DbError::SomethingWentWrong(e.to_string())),
                };
                db_error
            })?;

        let has_more = cpu_usage_logs.len() > limit;
        let messages: Vec<_> = cpu_usage_logs.into_iter().take(limit).collect();

        // Get next cursor from the last message's datetime
        let next_cursor = if has_more && !messages.is_empty() {
            Some(messages.last().unwrap().timestamp)
        } else {
            None
        };

        // Convert to DTOs
        let data: Vec<CpuUsageLogReadDto> = messages
            .into_iter()
            .map(CpuUsageLogReadDto::from)
            .collect();

        let pagination = PaginationMetadata {
            has_more,
            next_cursor,
            page_size: data.len(),
            total_count: None, // We don't calculate total count for cursor-based pagination
        };

        Ok(PaginatedCpuUsageLogResponse {
            data,
            pagination,
        })
    }
}

#[cfg(test)]
mod find_paginated_service_tests {
    use super::*;
    use crate::factory::cpu_usage_log_factory::CpuUsageLogFactory;
    use crate::repository::cpu_usage_log_repository::cpu_usage_log_repository_trait::MockCpuUsageLogRepositoryTrait;
    use bigdecimal::BigDecimal;
    use chrono::{Duration, Utc};
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_find_paginated_success_without_cursor() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let limit = 5;
        let pagination_query = CpuUsageLogFactory::fake_cpu_usage_log_pagination_query_with_limit(limit);
        let cpu_usage_logs = CpuUsageLogFactory::fake_cpu_usage_logs_paginated(limit);
        
        mock_repo
            .expect_find_paginated()
            .with(eq(None), eq(limit + 1))
            .times(1)
            .returning({
                let cpu_usage_logs = cpu_usage_logs.clone();
                move |_, _| Box::pin({
                    let cpu_usage_logs = cpu_usage_logs.clone();
                    async move { Ok(cpu_usage_logs) }
                })
            });

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.find_paginated_internal(pagination_query).await;

        // Assert
        assert!(result.is_ok(), "Failed to find paginated CPU usage logs: {:?}", result.err());
        let response = result.unwrap();
        assert_eq!(response.data.len(), limit);
        assert!(!response.pagination.has_more);
        assert!(response.pagination.next_cursor.is_none());
        assert_eq!(response.pagination.page_size, limit);
    }

    #[tokio::test]
    async fn test_find_paginated_success_with_cursor() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let limit = 3;
        let cursor = Utc::now() - Duration::hours(1);
        let pagination_query = CpuUsageLogFactory::fake_cpu_usage_log_pagination_query_with_cursor(cursor);
        let cpu_usage_logs = CpuUsageLogFactory::fake_cpu_usage_logs_paginated(limit);
        
        mock_repo
            .expect_find_paginated()
            .times(1)
            .returning({
                let cpu_usage_logs = cpu_usage_logs.clone();
                move |_, _| Box::pin({
                    let cpu_usage_logs = cpu_usage_logs.clone();
                    async move { Ok(cpu_usage_logs) }
                })
            });

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.find_paginated_internal(pagination_query).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.len(), limit);
        assert!(!response.pagination.has_more);
        assert!(response.pagination.next_cursor.is_none());
    }

    #[tokio::test]
    async fn test_find_paginated_success_with_more_data() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let limit = 3;
        let pagination_query = CpuUsageLogFactory::fake_cpu_usage_log_pagination_query_with_limit(limit);
        // Return limit + 1 items to simulate has_more = true
        let cpu_usage_logs = CpuUsageLogFactory::fake_cpu_usage_logs_paginated(limit + 1);
        let expected_next_cursor = cpu_usage_logs.get(limit - 1).unwrap().timestamp;
        
        mock_repo
            .expect_find_paginated()
            .with(eq(None), eq(limit + 1))
            .times(1)
            .returning({
                let cpu_usage_logs = cpu_usage_logs.clone();
                move |_, _| Box::pin({
                    let cpu_usage_logs = cpu_usage_logs.clone();
                    async move { Ok(cpu_usage_logs) }
                })
            });

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.find_paginated_internal(pagination_query).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.len(), limit); // Should only return 'limit' items, not limit+1
        assert!(response.pagination.has_more);
        assert!(response.pagination.next_cursor.is_some());
        assert_eq!(response.pagination.next_cursor.unwrap(), expected_next_cursor);
        assert_eq!(response.pagination.page_size, limit);
    }

    #[tokio::test]
    async fn test_find_paginated_empty_result() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let limit = 5;
        let pagination_query = CpuUsageLogFactory::fake_cpu_usage_log_pagination_query_with_limit(limit);
        
        mock_repo
            .expect_find_paginated()
            .with(eq(None), eq(limit + 1))
            .times(1)
            .returning(|_, _| Box::pin(async move { Ok(vec![]) }));

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.find_paginated_internal(pagination_query).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.len(), 0);
        assert!(!response.pagination.has_more);
        assert!(response.pagination.next_cursor.is_none());
        assert_eq!(response.pagination.page_size, 0);
    }

    #[tokio::test]
    async fn test_find_paginated_database_error() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let limit = 5;
        let pagination_query = CpuUsageLogFactory::fake_cpu_usage_log_pagination_query_with_limit(limit);
        
        mock_repo
            .expect_find_paginated()
            .with(eq(None), eq(limit + 1))
            .times(1)
            .returning(|_, _| Box::pin(async move { 
                Err(sqlx::Error::Configuration("Database connection failed".into())) 
            }));

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.find_paginated_internal(pagination_query).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::DbError(DbError::SomethingWentWrong(_))));
    }

    #[tokio::test]
    async fn test_find_paginated_single_item() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let limit = 5;
        let pagination_query = CpuUsageLogFactory::fake_cpu_usage_log_pagination_query_with_limit(limit);
        let cpu_usage_logs = CpuUsageLogFactory::fake_cpu_usage_logs_paginated(1);
        
        mock_repo
            .expect_find_paginated()
            .with(eq(None), eq(limit + 1))
            .times(1)
            .returning({
                let cpu_usage_logs = cpu_usage_logs.clone();
                move |_, _| Box::pin({
                    let cpu_usage_logs = cpu_usage_logs.clone();
                    async move { Ok(cpu_usage_logs) }
                })
            });

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.find_paginated_internal(pagination_query).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.len(), 1);
        assert!(!response.pagination.has_more);
        assert!(response.pagination.next_cursor.is_none());
        assert_eq!(response.pagination.page_size, 1);
    }

    #[tokio::test]
    async fn test_find_paginated_exact_limit() {
        // Arrange
        let mut mock_repo = MockCpuUsageLogRepositoryTrait::new();
        let limit = 3;
        let pagination_query = CpuUsageLogFactory::fake_cpu_usage_log_pagination_query_with_limit(limit);
        // Return exactly limit items (no more data)
        let cpu_usage_logs = CpuUsageLogFactory::fake_cpu_usage_logs_paginated(limit);
        
        mock_repo
            .expect_find_paginated()
            .with(eq(None), eq(limit + 1))
            .times(1)
            .returning({
                let cpu_usage_logs = cpu_usage_logs.clone();
                move |_, _| Box::pin({
                    let cpu_usage_logs = cpu_usage_logs.clone();
                    async move { Ok(cpu_usage_logs) }
                })
            });

        let service = CpuUsageLogService::new(Arc::new(mock_repo));

        // Act
        let result = service.find_paginated_internal(pagination_query).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.len(), limit);
        assert!(!response.pagination.has_more); // No more data because we got exactly limit items
        assert!(response.pagination.next_cursor.is_none());
        assert_eq!(response.pagination.page_size, limit);
    }
}
