use actix_web::{error::JsonPayloadError, web, HttpResponse};
use actix_web_validator::{
    error::flatten_errors, Error as JsonValidatorError, JsonConfig as JsonValidatorConfig,
};

use crate::models::custom_api_errors::ApiError;

fn json_payload_error_to_response(err: &JsonPayloadError) -> HttpResponse {
    let detail = err.to_string();

    match &err {
        JsonPayloadError::ContentType => {
            HttpResponse::UnsupportedMediaType().json(ApiError::BadRequest {
                message: "Unsupported media type".to_string(),
            })
        }
        JsonPayloadError::Deserialize(json_err) if json_err.is_data() => HttpResponse::BadRequest()
            .json(ApiError::BadRequest {
                message: json_err.to_string(),
            }),
        _ => HttpResponse::BadRequest().json(ApiError::BadRequest { message: detail }),
    }
}

pub fn json_config() -> web::JsonConfig {
    web::JsonConfig::default().error_handler(|err, _req| {
        let resp = json_payload_error_to_response(&err);

        actix_web::error::InternalError::from_response(err, resp).into()
    })
}

pub fn json_validator_config() -> JsonValidatorConfig {
    JsonValidatorConfig::default().error_handler(|err, _req| {
        let detail = err.to_string();

        let resp = match &err {
            JsonValidatorError::JsonPayloadError(err) => json_payload_error_to_response(err),
            JsonValidatorError::Validate(errors) => {
                let flattened_errors = flatten_errors(errors);

                if let Some(first_error) = flattened_errors.first() {
                    HttpResponse::BadRequest().json(ApiError::Validation {
                        message: first_error
                            .2
                            .message
                            .as_deref()
                            .unwrap_or("An unknown validation error occurred")
                            .to_string(),
                        code: first_error.2.code.to_string(),
                        field: Some(first_error.1.clone()),
                    })
                } else {
                    HttpResponse::BadRequest().json(ApiError::Validation {
                        message: "An unknown validation error occurred".to_string(),
                        code: "unknown".to_string(),
                        field: None,
                    })
                }
            }
            _ => HttpResponse::BadRequest().json(ApiError::Validation {
                message: detail,
                code: "unknown".to_string(),
                field: None,
            }),
        };

        actix_web::error::InternalError::from_response(err, resp).into()
    })
}

pub fn cors_config() -> actix_cors::Cors {
    actix_cors::Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header()
}
