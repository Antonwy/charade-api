use actix_session::{SessionGetError, SessionInsertError};
use actix_web::{
    error::{BlockingError, ResponseError},
    http::StatusCode,
};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use redis::RedisError;
use serde::{Deserialize, Serialize};

pub type Result<T, E = ApiError> = std::result::Result<T, E>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ApiError {
    Internal {
        message: String,
    },
    BadRequest {
        message: String,
    },
    NotFound {
        message: String,
    },
    Unauthorized {
        message: String,
    },
    UniqueViolation {
        message: String,
    },
    CheckViolation {
        message: String,
    },
    ForeignKeyViolation {
        message: String,
    },
    NotNullViolation {
        message: String,
    },
    Validation {
        message: String,
        code: String,
        field: Option<String>,
    },
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApiError::Internal { message } => {
                write!(f, "Internal error: {} - {}", self.status_code(), message)
            }
            ApiError::BadRequest { message } => {
                write!(f, "Bad request: {} - {}", self.status_code(), message)
            }
            ApiError::NotFound { message } => {
                write!(f, "Not found: {} - {}", self.status_code(), message)
            }
            ApiError::Unauthorized { message } => {
                write!(f, "Unauthorized: {} - {}", self.status_code(), message)
            }
            ApiError::UniqueViolation { message } => {
                write!(f, "Unique violation: {} - {}", self.status_code(), message)
            }
            ApiError::Validation { message, .. } => {
                write!(f, "Validation error: {} - {}", self.status_code(), message)
            }
            ApiError::CheckViolation { message } => {
                write!(f, "Check violation: {} - {}", self.status_code(), message)
            }
            ApiError::ForeignKeyViolation { message } => {
                write!(
                    f,
                    "Foreign key violation: {} - {}",
                    self.status_code(),
                    message
                )
            }
            ApiError::NotNullViolation { message } => {
                write!(
                    f,
                    "Not null violation: {} - {}",
                    self.status_code(),
                    message
                )
            }
        }
    }
}

impl Default for ApiError {
    fn default() -> Self {
        ApiError::Internal {
            message: "Internal server error".to_string(),
        }
    }
}

impl From<DieselError> for ApiError {
    fn from(error: DieselError) -> ApiError {
        log::warn!("Diesel error: {}", error);

        match error {
            DieselError::DatabaseError(kind, _) => match kind {
                DatabaseErrorKind::UniqueViolation => ApiError::UniqueViolation {
                    message: "It seems like this record already exists".to_string(),
                },
                DatabaseErrorKind::CheckViolation => ApiError::CheckViolation {
                    message: "Constraint check failed".to_string(),
                },
                DatabaseErrorKind::ForeignKeyViolation => ApiError::ForeignKeyViolation {
                    message: "Foreign key constraint check failed".to_string(),
                },
                DatabaseErrorKind::NotNullViolation => ApiError::NotNullViolation {
                    message: "Not null constraint check failed".to_string(),
                },
                _ => ApiError::default(),
            },
            DieselError::NotFound => ApiError::NotFound {
                message: "Record not found".to_string(),
            },
            _ => ApiError::default(),
        }
    }
}

impl From<SessionInsertError> for ApiError {
    fn from(error: SessionInsertError) -> ApiError {
        ApiError::Internal {
            message: format!("Session error: {}", error),
        }
    }
}

impl From<SessionGetError> for ApiError {
    fn from(_: SessionGetError) -> ApiError {
        ApiError::Unauthorized {
            message: "Please authenticate first".to_string(),
        }
    }
}

impl From<BlockingError> for ApiError {
    fn from(error: BlockingError) -> ApiError {
        ApiError::Internal {
            message: format!("Blocking error: {}", error),
        }
    }
}

impl From<RedisError> for ApiError {
    fn from(error: RedisError) -> ApiError {
        ApiError::Internal {
            message: format!("Redis error: {}", error),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> ApiError {
        ApiError::Internal {
            message: format!("Serde error: {}", error),
        }
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(error: validator::ValidationErrors) -> ApiError {
        ApiError::BadRequest {
            message: error.to_string(),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code()).json(self)
    }
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            ApiError::UniqueViolation { .. } => StatusCode::BAD_REQUEST,
            ApiError::Validation { .. } => StatusCode::BAD_REQUEST,
            ApiError::CheckViolation { .. } => StatusCode::BAD_REQUEST,
            ApiError::ForeignKeyViolation { .. } => StatusCode::BAD_REQUEST,
            ApiError::NotNullViolation { .. } => StatusCode::BAD_REQUEST,
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
        }
    }

    pub fn internal(message: String) -> ApiError {
        ApiError::Internal { message }
    }
}
