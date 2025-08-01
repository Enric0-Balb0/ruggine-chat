use crate::http::error::HttpError;
use crate::types::common::{ApiErrorResponse, ApiResponse};
use crate::config::constants::AppConstants;
use reqwest::{Client, RequestBuilder, Response};
use serde::{de::DeserializeOwned, Serialize};

/// Client HTTP centralizzato per tutte le chiamate API
#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    auth_token: Option<String>,
}

impl ApiClient {
    /// Crea un nuovo client API
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.into(),
            auth_token: None,
        }
    }

    /// Imposta il token di autenticazione
    pub fn with_auth_token(mut self, token: Option<String>) -> Self {
        self.auth_token = token;
        self
    }

    /// Aggiorna il token di autenticazione
    pub fn set_auth_token(&mut self, token: Option<String>) {
        self.auth_token = token;
    }

    /// Ottiene il token di autenticazione corrente
    pub fn auth_token(&self) -> Option<&String> {
        self.auth_token.as_ref()
    }

    /// Costruisce l'URL completo per un endpoint
    fn build_url(&self, endpoint: &str) -> String {
        format!("{}/api{}", self.base_url.trim_end_matches('/'), endpoint)
    }

    /// Aggiunge headers comuni alla request
    fn add_common_headers(&self, builder: RequestBuilder) -> RequestBuilder {
        let mut builder = builder
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        if let Some(token) = &self.auth_token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }

        builder
    }

    /// Gestisce la risposta HTTP e deserializza
    async fn handle_response<T: DeserializeOwned>(response: Response) -> Result<T, HttpError> {
        let status = response.status();
        let text = response.text().await?;

        if status.is_success() {
            // Prova prima a deserializzare come ApiResponse<T>
            if let Ok(api_response) = serde_json::from_str::<ApiResponse<T>>(&text) {
                Ok(api_response.data)
            } else {
                // Fallback: deserializza direttamente come T
                serde_json::from_str(&text).map_err(|e| {
                    HttpError::Deserialization(format!("Failed to parse response: {}", e))
                })
            }
        } else {
            // Prova a parsare come errore strutturato
            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&text) {
                Err(HttpError::Http {
                    status: status.as_u16(),
                    message: error_response.message,
                })
            } else {
                Err(HttpError::Http {
                    status: status.as_u16(),
                    message: text,
                })
            }
        }
    }

    /// GET request
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, HttpError> {
        let url = self.build_url(endpoint);
        let request = self.add_common_headers(self.client.get(&url));
        let response = request.send().await?;
        Self::handle_response(response).await
    }

    /// POST request con body
    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, HttpError> {
        let url = self.build_url(endpoint);
        let request = self.add_common_headers(self.client.post(&url));
        let response = request.json(body).send().await?;
        Self::handle_response(response).await
    }

    /// PUT request con body
    pub async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, HttpError> {
        let url = self.build_url(endpoint);
        let request = self.add_common_headers(self.client.put(&url));
        let response = request.json(body).send().await?;
        Self::handle_response(response).await
    }

    /// DELETE request
    pub async fn delete<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, HttpError> {
        let url = self.build_url(endpoint);
        let request = self.add_common_headers(self.client.delete(&url));
        let response = request.send().await?;
        Self::handle_response(response).await
    }

    /// PATCH request con body
    pub async fn patch<B: Serialize, T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, HttpError> {
        let url = self.build_url(endpoint);
        let request = self.add_common_headers(self.client.patch(&url));
        let response = request.json(body).send().await?;
        Self::handle_response(response).await
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        // URL di default - andrà configurato via environment o config
        Self::new(AppConstants::DEFAULT_SERVER_URL)
    }
}
