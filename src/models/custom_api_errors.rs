use actix_session::{SessionGetError, SessionInsertError};
use actix_web::error::ResponseError;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use serde::{Deserialize, Serialize};

pub type Result<T, E = ApiError> = std::result::Result<T, E>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ApiError: {} - {}", self.status, self.message)
    }
}

impl From<DieselError> for ApiError {
    fn from(error: DieselError) -> ApiError {
        match error {
            DieselError::DatabaseError(kind, err) => match kind {
                DatabaseErrorKind::UniqueViolation => {
                    ApiError::bad_request("This information already exists".to_string())
                }
                _ => ApiError::conflict(err.message().to_string()),
            },
            DieselError::NotFound => ApiError::not_found("Record not found".to_string()),
            err => ApiError::internal(format!("Diesel error: {}", err)),
        }
    }
}

impl From<SessionInsertError> for ApiError {
    fn from(error: SessionInsertError) -> ApiError {
        ApiError::internal(format!("Session error: {}", error))
    }
}

impl From<SessionGetError> for ApiError {
    fn from(_: SessionGetError) -> ApiError {
        ApiError::unauthorized("Please authenticate first".to_string())
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::from_u16(self.status).unwrap()
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code()).json(self)
    }
}

impl ApiError {
    pub fn new(status: u16, message: String) -> ApiError {
        ApiError { status, message }
    }

    pub fn conflict(message: String) -> Self {
        Self {
            status: 409,
            message,
        }
    }

    pub fn internal(message: String) -> Self {
        Self {
            status: 500,
            message,
        }
    }

    pub fn not_found(message: String) -> Self {
        Self {
            status: 404,
            message,
        }
    }

    pub fn bad_request(message: String) -> Self {
        Self {
            status: 400,
            message,
        }
    }

    pub fn unauthorized(message: String) -> Self {
        Self {
            status: 401,
            message,
        }
    }

    pub fn forbidden(message: String) -> Self {
        Self {
            status: 403,
            message,
        }
    }
}
