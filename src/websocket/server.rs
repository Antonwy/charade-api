use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix::{Actor, Addr, Context};
use actix_web::web;

use crate::{
    models::{user::User, word::NewWord},
    repositories::{cache::Cache, database::Database},
    websocket::{messages::ServerMessage, session::WsCharadeSession},
};

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

        let session_id_clone = session_id.to_owned();
        let word_clone = word.to_owned();
        let user_id_clone = user_id.to_owned();

        let error = ServerError::Private {
            id: user_id.to_string(),
            error: "Could not add word to session".to_string(),
        };

        let block_res = web::block(move || {
            db.add_word_to_session(NewWord {
                session_id: session_id_clone.clone(),
                word: word_clone,
                user_id: user_id_clone,
            })?;
            db.get_words_by_session_id(&session_id_clone)
        })
        .await;

        let Ok(words_res) = block_res else {
            log::error!("Could not add word to session: {:?}", block_res.err());
            return Err(error);
        };

        let Ok(words) = words_res else {
            log::error!("Could not add word to session: {:?}", words_res.err());
            return Err(ServerError::Private { id: user_id.to_string(), error: format!("Word '{word}' already in session") });
        };

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
        let session_id_clone = session_id.to_owned();

        let error = ServerError::None;

        let block_res = web::block(move || db.get_users_by_session_id(&session_id_clone)).await;

        let Ok(session_res) = block_res else {
            log::error!("Could get session users: {:?}", block_res.err());
            return Err(error);
        };

        let Ok(session_users) = session_res else {
            log::error!("Could get session users: {:?}", session_res.err());
            return Err(error);
        };

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
