use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    models::{user::User, word::NewWord},
    repositories::{cache::Cache, database::Database},
    websocket::{messages::ServerMessage, session::WsCharadeSession},
};
use actix::{Actor, Addr, Context};

use self::utils::{Result, ServerError, ServerResult};

mod handlers;
mod utils;

#[derive(Debug, Clone)]
pub struct CharadeServer {
    sessions: Arc<Mutex<HashMap<String, Addr<WsCharadeSession>>>>,
    db: Database,
    cache: Cache,
}

impl CharadeServer {
    pub fn new(db: Database, cache: Cache) -> CharadeServer {
        CharadeServer {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            db,
            cache,
        }
    }

    pub async fn add_word_to_session(&self, session_id: &str, word: &str, user_id: &str) -> Result {
        let db = self.db.clone();

        db.add_word_to_session(NewWord {
            session_id: session_id.to_string(),
            word: word.to_string(),
            user_id: user_id.to_string(),
        })
        .await
        .map_err(|_| ServerError::Private {
            id: user_id.to_string(),
            error: format!("Word '{word}' already in session"),
        })?;

        let words =
            db.get_words_by_session_id(session_id)
                .await
                .map_err(|_| ServerError::Private {
                    id: user_id.to_string(),
                    error: format!("Word '{word}' already in session"),
                })?;

        Ok(ServerResult::Multiple(vec![
            ServerResult::Broadcast {
                session_id: session_id.to_string(),
                msg: ServerMessage::AddWord {
                    number_of_words: words.len() as u16,
                },
                exclude: Some(user_id.to_string()),
            },
            ServerResult::Private {
                id: user_id.to_string(),
                msg: ServerMessage::AddWordPersonal {
                    number_of_words: words.len() as u16,
                    my_words: words
                        .iter()
                        .filter(|w| w.user_id == user_id)
                        .map(|w| w.word.clone())
                        .collect(),
                },
            },
        ]))
    }

    pub async fn handle_update_users(&self, session_id: &str) -> Result<ServerMessage> {
        let db = self.db.clone();

        let session_users = db
            .get_users_by_session_id(session_id)
            .await
            .map_err(|_| ServerError::None)?;

        let online_users: Vec<User> = session_users
            .iter()
            .filter(|u| self.user_id_in_current_sessions(&u.id))
            .cloned()
            .collect();

        let offline_users: Vec<User> = session_users
            .iter()
            .filter(|u| !self.user_id_in_current_sessions(&u.id))
            .cloned()
            .collect();

        Ok(ServerMessage::UsersUpdate {
            online_users,
            offline_users,
        })
    }
}

impl Actor for CharadeServer {
    type Context = Context<Self>;
}
