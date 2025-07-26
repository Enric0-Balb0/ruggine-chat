use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct ApiSuccessResponse<T: Serialize> {
    data: T,
}

impl<T: Serialize> ApiSuccessResponse<T>
where
    T: Serialize + for<'s> ToSchema<'s>,
{
    pub(crate) fn send(data: T) -> Self {
        return ApiSuccessResponse { data };
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ApiErrorResponse {
    message: Option<String>,
    #[serde(rename = "code")]
    status: u16,
}

impl ApiErrorResponse {
    pub(crate) fn send(status: u16, message: Option<String>) -> Response {
        return ApiErrorResponse { message, status }.into_response();
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        (
            StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(self),
        )
            .into_response()
    }
}
