use actix::Addr;
use actix_web::web;

use crate::{
    models::custom_api_errors::ApiError,
    repositories::cache::Cache,
    websocket::{
        messages::ServerMessage, server::CharadeServer, session::WsCharadeSession, ClientMessage,
    },
};

#[async_trait::async_trait]
pub trait ServerMessageHandler {
    async fn distribute_message(&self, server: &CharadeServer);
}

#[derive(Debug)]
pub enum ServerResult {
    None,
    Private {
        id: String,
        msg: ServerMessage,
    },
    Broadcast {
        session_id: String,
        msg: ServerMessage,
        exclude: Option<String>,
    },
    Multiple(Vec<ServerResult>),
}

#[async_trait::async_trait]
impl ServerMessageHandler for ServerResult {
    async fn distribute_message(&self, server: &CharadeServer) {
        match self {
            ServerResult::None => {}
            ServerResult::Private { id, msg } => server.send(id, msg.clone()),
            ServerResult::Broadcast {
                session_id,
                msg,
                exclude,
            } => {
                let res = server
                    .broadcast_session(session_id, msg.clone(), exclude.clone())
                    .await;

                if let Err(err) = res {
                    log::error!("Could not broadcast message: {}", err);
                }
            }
            ServerResult::Multiple(results) => {
                for result in results {
                    result.distribute_message(server).await;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ServerError {
    Private {
        id: String,
        error: String,
    },
    #[allow(dead_code)]
    Broadcast {
        session_id: String,
        error: String,
    },
    None,
}

#[async_trait::async_trait]
impl ServerMessageHandler for ServerError {
    async fn distribute_message(&self, server: &CharadeServer) {
        match self {
            ServerError::Private { id, error } => server.send(
                id,
                ServerMessage::Error {
                    error: error.to_string(),
                },
            ),
            ServerError::Broadcast { session_id, error } => {
                let res = server
                    .broadcast_session(
                        session_id,
                        ServerMessage::Error {
                            error: error.to_string(),
                        },
                        None,
                    )
                    .await;

                if let Err(err) = res {
                    log::error!("Could not broadcast message: {}", err);
                }
            }
            ServerError::None => {
                log::error!("Could not distribute message: {:?}", self);
            }
        }
    }
}

pub type Result<T = ServerResult, E = ServerError> = std::result::Result<T, E>;

impl CharadeServer {
    fn send(&self, id: &str, msg: ServerMessage) {
        let client_addr = self.get_user_session_addr(id);

        if let Some(addr) = client_addr {
            addr.do_send(msg);
        }
    }

    fn broadcast(&self, ids: Vec<&str>, msg: ServerMessage) {
        for id in ids {
            self.send(id, msg.clone());
        }
    }

    async fn broadcast_session(
        &self,
        session_id: &str,
        msg: ServerMessage,
        exclude: Option<String>,
    ) -> Result<(), ApiError> {
        let session_id = session_id.to_owned();

        let session_ids = self.get_cached_session_users(&session_id).await;

        self.broadcast(
            session_ids
                .iter()
                .filter(|id| {
                    if let Some(exclude) = &exclude {
                        return *id != exclude;
                    }

                    true
                })
                .filter(|id| self.user_id_in_current_sessions(id))
                .map(|id| id.as_str())
                .collect(),
            msg,
        );

        Ok(())
    }

    pub async fn handle_incoming_client_message(
        &self,
        msg: ClientMessage,
        client_id: &str,
        session_id: &str,
    ) -> Result {
        match msg {
            ClientMessage::StartSession { session_id } => {
                println!("Starting session: {session_id}");
                Ok(ServerResult::None)
            }
            ClientMessage::AddWord { word } => {
                println!("Adding word: {word} to session: {session_id} for user: {client_id}");
                self.add_word_to_session(session_id, &word, client_id).await
            }
        }
    }

    async fn get_cached_session_users(&self, session_id: &str) -> Vec<String> {
        let res = self
            .cache
            .get_string_set(&Cache::session_users_key(session_id))
            .await;

        let ids = match res {
            Ok(users) => users,
            Err(_) => {
                let db = self.db.clone();
                let session_id_owned = session_id.to_owned();

                let blocking_res =
                    web::block(move || db.get_users_by_session_id(&session_id_owned)).await;

                let Ok(session_res) = blocking_res else {
                        log::error!("Could not get session users: {:?}", blocking_res.err());
                        return vec![];
                    };

                let Ok(session_users) = session_res else {
                        log::error!("Could not get session users: {:?}", session_res.err());
                        return vec![];
                    };

                let cache_res = self
                    .cache
                    .push_strings_to_set(
                        &Cache::session_users_key(session_id),
                        session_users.iter().map(|u| u.id.as_str()).collect(),
                    )
                    .await;

                if let Err(err) = cache_res {
                    log::error!("Could not cache session users: {}", err);
                }

                session_users.iter().map(|u| u.id.clone()).collect()
            }
        };

        ids.iter()
            .filter(|id| self.user_id_in_current_sessions(id))
            .cloned()
            .collect()
    }

    pub fn user_id_in_current_sessions(&self, user_id: &str) -> bool {
        let sessions_lock = self.sessions.lock().unwrap();

        sessions_lock.contains_key(user_id)
    }

    pub fn get_user_session_addr(&self, user_id: &str) -> Option<Addr<WsCharadeSession>> {
        let sessions_lock = self.sessions.lock().unwrap();

        let addr = sessions_lock.get(user_id);

        addr.cloned()
    }
}
