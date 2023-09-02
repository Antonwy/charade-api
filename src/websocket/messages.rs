use actix::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::{custom_api_errors::ApiError, session::Session, user::User};

use super::session::WsCharadeSession;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(String)]
pub struct Connect {
    pub addr: Addr<WsCharadeSession>,
    pub id: String,
    pub session_id: String,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
    pub session_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessageWrapper {
    pub id: String,
    pub session_id: String,
    pub message: ClientMessage,
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    StartSession { session_id: String },
    AddWord { word: String },
}

#[derive(Message, Debug, Serialize, Clone)]
#[serde(tag = "type", content = "payload")]
#[rtype(result = "()")]
pub enum ServerMessage {
    UsersUpdate {
        online_users: Vec<User>,
        offline_users: Vec<User>,
    },
    AddWord {
        number_of_words: u16,
    },
    AddWordPersonal {
        number_of_words: u16,
        my_words: Vec<String>,
    },
    Error {
        error: String,
    },
    None,
}

impl From<ApiError> for ServerMessage {
    fn from(error: ApiError) -> ServerMessage {
        match error {
            ApiError::Internal { message }
            | ApiError::BadRequest { message }
            | ApiError::NotFound { message }
            | ApiError::Unauthorized { message }
            | ApiError::UniqueViolation { message }
            | ApiError::Validation { message, .. }
            | ApiError::CheckViolation { message }
            | ApiError::ForeignKeyViolation { message }
            | ApiError::NotNullViolation { message } => ServerMessage::Error { error: message },
        }
    }
}
