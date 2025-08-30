use crate::api::client::ApiClient;
use crate::utils::storage::StorageService;
use crate::error::AuthError;
use crate::config::endpoints::ApiEndpoints;
use crate::types::cpu_usage_log::{PaginatedCpuUsageLogResponse, CpuUsagePage};
// debug imports removed

#[derive(Clone)]
pub struct CpuUsageLogService {
    http_client: ApiClient,
    storage_service: StorageService,
}

impl CpuUsageLogService {
    pub fn new(http_client: ApiClient, storage_service: StorageService) -> Self {
        Self { http_client, storage_service }
    }

    /// Retrieve paginated CPU usage logs. Cursor is an optional RFC3339 timestamp.
    pub async fn find_paginated(
        &self,
        cursor: Option<String>,
        limit: Option<i32>,
    ) -> Result<CpuUsagePage, AuthError> {
        let mut url = ApiEndpoints::CPU_USAGE_LOG.to_string();
        let mut params = vec![];
        if let Some(c) = cursor {
            params.push(("cursor", c));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }
        if !params.is_empty() {
            let query = serde_urlencoded::to_string(&params).unwrap();
            url = format!("{}?{}", url, query);
        }
        let resp: PaginatedCpuUsageLogResponse = self.http_client.get(&url).await.map_err(AuthError::from)?;

    Ok(CpuUsagePage::from(resp))
    }
}
