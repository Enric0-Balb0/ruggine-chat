use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Generic paginated response for cursor-based pagination
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct PaginatedResponse<T> {
    /// The data items for this page
    #[schema(example = "[]")]
    pub data: Vec<T>,
    
    /// Information about pagination
    pub pagination: PaginationInfo,
}

/// Pagination metadata for cursor-based pagination
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct PaginationInfo {
    /// Whether there are more items available
    #[schema(example = true)]
    pub has_more: bool,
    
    /// Cursor for the next page (timestamp of the last item in current page)
    /// Use this as 'before' parameter for the next request
    #[schema(example = "2025-08-12T10:30:00Z")]
    pub next_cursor: Option<DateTime<Utc>>,
    
    /// Number of items in current page
    #[schema(example = 20)]
    pub count: usize,
    
    /// Maximum items per page
    #[schema(example = 20)]
    pub page_size: usize,
}

/// Query parameters for cursor-based pagination
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate, PartialEq, Eq)]
pub struct PaginationQuery {
    /// Get messages before this timestamp (for loading older messages)
    /// If not provided, gets the most recent messages
    #[schema(example = "2025-08-12T10:30:00Z")]
    pub before: Option<DateTime<Utc>>,
    
    /// Number of items to return (max 100)
    #[validate(range(min = 1, max = 100, message = "Page size must be between 1 and 100"))]
    #[schema(example = 20)]
    pub limit: Option<usize>,
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            before: None,
            limit: Some(20), // Default page size
        }
    }
}

impl PaginationQuery {
    pub fn limit(&self) -> usize {
        self.limit.unwrap_or(20).min(100)
    }
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, has_more: bool, next_cursor: Option<DateTime<Utc>>, page_size: usize) -> Self {
        let count = data.len();
        
        Self {
            data,
            pagination: PaginationInfo {
                has_more,
                next_cursor,
                count,
                page_size,
            },
        }
    }
    
    /// Create empty paginated response
    pub fn empty(page_size: usize) -> Self {
        Self::new(vec![], false, None, page_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_pagination_query_default() {
        let query = PaginationQuery::default();
        assert_eq!(query.before, None);
        assert_eq!(query.limit(), 20);
    }

    #[test]
    fn test_pagination_query_limit_clamping() {
        let query = PaginationQuery {
            before: None,
            limit: Some(150), // Over max
        };
        assert_eq!(query.limit(), 100); // Should be clamped to max

        let query = PaginationQuery {
            before: None,
            limit: None,
        };
        assert_eq!(query.limit(), 20); // Should use default
    }

    #[test]
    fn test_paginated_response_creation() {
        let data = vec!["item1".to_string(), "item2".to_string()];
        let next_cursor = Some(Utc::now());
        let response = PaginatedResponse::new(data.clone(), true, next_cursor, 20);

        assert_eq!(response.data, data);
        assert_eq!(response.pagination.count, 2);
        assert_eq!(response.pagination.page_size, 20);
        assert_eq!(response.pagination.has_more, true);
        assert_eq!(response.pagination.next_cursor, next_cursor);
    }

    #[test]
    fn test_empty_paginated_response() {
        let response: PaginatedResponse<String> = PaginatedResponse::empty(20);
        
        assert_eq!(response.data.len(), 0);
        assert_eq!(response.pagination.count, 0);
        assert_eq!(response.pagination.page_size, 20);
        assert_eq!(response.pagination.has_more, false);
        assert_eq!(response.pagination.next_cursor, None);
    }

    #[test]
    fn test_pagination_query_validation() {
        use validator::Validate;

        let valid_query = PaginationQuery {
            before: None,
            limit: Some(50),
        };
        assert!(valid_query.validate().is_ok());

        let invalid_query = PaginationQuery {
            before: None,
            limit: Some(150), // Over max
        };
        assert!(invalid_query.validate().is_err());

        let invalid_query = PaginationQuery {
            before: None,
            limit: Some(0), // Under min
        };
        assert!(invalid_query.validate().is_err());
    }
}
