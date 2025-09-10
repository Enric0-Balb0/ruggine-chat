use crate::api::http_error::HttpError;
use crate::types::common::{ApiErrorResponse, ApiResponse, ApiSuccessResponse};
use crate::config::constants::AppConstants;
use crate::utils::storage::StorageService;
use reqwest::{Client, RequestBuilder, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{Arc, Mutex};
use web_sys;

/// Client HTTP centralizzato per tutte le chiamate API
#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    auth_token: Arc<Mutex<Option<String>>>,
    storage_service: StorageService,
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
            auth_token: Arc::new(Mutex::new(None)),
            storage_service: StorageService::new(),
        }
    }

    /// Imposta il token di autenticazione
    pub fn with_auth_token(self, token: Option<String>) -> Self {
        if let Ok(mut auth_token) = self.auth_token.lock() {
            *auth_token = token;
        }
        self
    }

    /// Aggiorna il token di autenticazione
    pub fn set_auth_token(&self, token: Option<String>) {
        if let Ok(mut auth_token) = self.auth_token.lock() {
            *auth_token = token;
        }
    }

    /// Ottiene il token di autenticazione corrente
    pub fn auth_token(&self) -> Option<String> {
        self.auth_token.lock().ok().and_then(|guard| guard.clone())
    }

    /// Costruisce l'URL completo per un endpoint
    fn build_url(&self, endpoint: &str) -> String {
        format!("{}/api{}", self.base_url.trim_end_matches('/'), endpoint)
    }

    /// Aggiunge headers comuni alla request
    fn add_common_headers(&self, builder: RequestBuilder) -> RequestBuilder {
        let mut builder = builder
            // Do not set Content-Type globally: adding Content-Type to GET requests
            // forces browsers to perform a CORS preflight (OPTIONS). We only set
            // Accept here and rely on `RequestBuilder::json()` to add the proper
            // Content-Type for requests with a body (POST/PUT/PATCH).
            .header("Accept", "application/json");

        if let Ok(auth_token) = self.auth_token.lock() {
            if let Some(token) = auth_token.as_ref() {
                builder = builder.header("Authorization", format!("Bearer {}", token));
            }
        }

        builder
    }

    /// Gestisce la risposta HTTP e deserializza
    /// Automaticamente gestisce il logout su 401 Unauthorized
    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T, HttpError> {
    let status = response.status();
    let text = response.text().await?;
    // ...

        if status.is_success() {
            // Prova prima a deserializzare come ApiResponse<T> (formato client)
            if let Ok(api_response) = serde_json::from_str::<ApiResponse<T>>(&text) {
                Ok(api_response.data)
            }
            // Poi prova come ApiSuccessResponse<T> (formato server)
            else if let Ok(server_response) = serde_json::from_str::<ApiSuccessResponse<T>>(&text) {
                Ok(server_response.data)
            }
            // Fallback: deserializza direttamente come T
            else {
                serde_json::from_str(&text).map_err(|e| {
                    HttpError::Deserialization(format!("Failed to parse response: {}", e))
                })
            }
        } else {
            // Gestione speciale per 401 Unauthorized
            if status.as_u16() == 401 {
                self.handle_unauthorized().await;
            }
            
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

    /// Gestisce automaticamente il caso 401 Unauthorized
    /// Pulisce la sessione e reindirizza al login
    async fn handle_unauthorized(&self) {
        // Pulisci il token dal client
        if let Ok(mut auth_token) = self.auth_token.lock() {
            *auth_token = None;
        }
        
        // Pulisci tutto il localStorage
        self.storage_service.clear_all();
        
        // Reindirizza al login usando window.location (più robusto)
        leptos::spawn_local(async {
            if let Some(window) = web_sys::window() {
                let location = window.location();
                if let Err(e) = location.set_href("/login") {
                    leptos::logging::error!("Cannot redirect to login: {:?}", e);
                }
            } else {
                leptos::logging::error!("Cannot access window for redirect");
            }
        });
    }

    /// GET request
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, HttpError> {
        let url = self.build_url(endpoint);
        let request = self.add_common_headers(self.client.get(&url));
        let response = request.send().await?;
        self.handle_response(response).await
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
        self.handle_response(response).await
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
        self.handle_response(response).await
    }

    /// DELETE request
    pub async fn delete<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, HttpError> {
        let url = self.build_url(endpoint);
        let request = self.add_common_headers(self.client.delete(&url));
        let response = request.send().await?;
        self.handle_response(response).await
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
        self.handle_response(response).await
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        // URL di default - andrà configurato via environment o config
        Self::new(AppConstants::DEFAULT_SERVER_URL)
    }
}
