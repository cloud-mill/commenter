use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub struct ServerError {
    pub message: Option<String>,
    pub status_code: StatusCode,
}

impl ServerError {
    pub fn internal_server_error() -> ServerError {
        ServerError {
            message: None,
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn forbidden_error() -> ServerError {
        ServerError {
            message: None,
            status_code: StatusCode::FORBIDDEN,
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        if self.message.is_some() {
            (
                self.status_code,
                Json(json!({
                  "error": self.message
                })),
            )
                .into_response()
        } else {
            self.status_code.into_response()
        }
    }
}
