use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::text_message_dto::TextMessageReadDto;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct PaginatedResponse<T, C> {
    /// The data items for this page
    #[schema(example = "[]")]
    pub data: Vec<T>,
    
    /// Pagination metadata
    pub pagination: PaginationMetadata<C>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct PaginationMetadata<C> {
    /// Whether there are more items available
    #[schema(example = true)]
    pub has_more: bool,
    
    /// Cursor for the next page (timestamp or ID)
    #[schema(example = "2025-08-12T10:30:00Z")]
    pub next_cursor: Option<C>,
    
    /// Number of items in this page
    #[schema(example = 20)]
    pub page_size: usize,
    
    /// Total count (optional, can be expensive to compute)
    #[schema(example = 150)]
    pub total_count: Option<usize>,
}

impl<T, C> PaginatedResponse<T, C> {
    pub fn new(
        data: Vec<T>,
        has_more: bool,
        next_cursor: Option<C>,
        total_count: Option<usize>,
    ) -> Self {
        let page_size = data.len();
        Self {
            data,
            pagination: PaginationMetadata {
                has_more,
                next_cursor,
                page_size,
                total_count,
            },
        }
    }

    /// Create a response for the last page
    pub fn last_page(data: Vec<T>) -> Self {
        let page_size = data.len();
        Self {
            data,
            pagination: PaginationMetadata {
                has_more: false,
                next_cursor: None,
                page_size,
                total_count: None,
            },
        }
    }

    /// Create an empty response
    pub fn empty() -> Self {
        Self {
            data: Vec::new(),
            pagination: PaginationMetadata {
                has_more: false,
                next_cursor: None,
                page_size: 0,
                total_count: Some(0),
            },
        }
    }
}

pub type PaginationMetadataDateTime = PaginationMetadata<DateTime<Utc>>;
pub type PaginatedTextMessageResponse = PaginatedResponse<TextMessageReadDto, DateTime<Utc>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginated_response_new() {
        let data = vec!["item1", "item2", "item3"];
        let response = PaginatedResponse::new(
            data.clone(),
            true,
            Some("cursor123".to_string()),
            Some(100),
        );

        assert_eq!(response.data, data);
        assert_eq!(response.pagination.has_more, true);
        assert_eq!(response.pagination.next_cursor, Some("cursor123".to_string()));
        assert_eq!(response.pagination.page_size, 3);
        assert_eq!(response.pagination.total_count, Some(100));
    }

    #[test]
    fn test_paginated_response_last_page() {
        let data = vec!["item1", "item2"];
        let response: PaginatedResponse<&str, String> = PaginatedResponse::last_page(data.clone());

        assert_eq!(response.data, data);
        assert_eq!(response.pagination.has_more, false);
        assert_eq!(response.pagination.next_cursor, None);
        assert_eq!(response.pagination.page_size, 2);
    }

    #[test]
    fn test_paginated_response_empty() {
        let response: PaginatedResponse<String, String> = PaginatedResponse::empty();

        assert_eq!(response.data.len(), 0);
        assert_eq!(response.pagination.has_more, false);
        assert_eq!(response.pagination.next_cursor, None);
        assert_eq!(response.pagination.page_size, 0);
        assert_eq!(response.pagination.total_count, Some(0));
    }

}
