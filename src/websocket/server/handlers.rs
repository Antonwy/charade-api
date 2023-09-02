pub mod handlers {
    use actix::{Context, Handler, ResponseFuture};

    use crate::{
        repositories::cache::Cache,
        websocket::{
            messages::{ClientMessageWrapper, Connect, Disconnect},
            server::{
                utils::utils::{ServerMessageHandler, ServerResult},
                CharadeServer,
            },
        },
    };

    impl Handler<Connect> for CharadeServer {
        type Result = ResponseFuture<String>;

        fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
            println!("Someone joined");

            let id = msg.id.clone();

            let mut session_lock = self.sessions.lock().unwrap();

            session_lock.insert(msg.id.clone(), msg.addr);

            let cache_key = Cache::session_users_key(&msg.session_id);

            let this = self.clone();
            let user_id = msg.id.clone();

            Box::pin(async move {
                let _ = this
                    .cache
                    .push_strings_to_set(&cache_key, vec![&user_id])
                    .await;

                let res = this.handle_update_users(&msg.session_id).await;

                if let Ok(res) = res {
                    ServerResult::Broadcast {
                        session_id: msg.session_id.to_string(),
                        msg: res,
                        exclude: None,
                    }
                    .distribute_message(&this)
                    .await;
                }

                id
            })
        }
    }

    impl Handler<Disconnect> for CharadeServer {
        type Result = ResponseFuture<()>;

        fn handle(&mut self, msg: Disconnect, ctx: &mut Context<Self>) -> Self::Result {
            let cache_key = Cache::session_users_key(&msg.session_id);

            let user_id = msg.id.clone();

            let mut sessions_lock = self.sessions.lock().unwrap();

            sessions_lock.remove(&user_id);

            let this = self.clone();

            Box::pin(async move {
                let _ = this
                    .cache
                    .remove_string_from_set(&cache_key, &user_id)
                    .await;

                let res = this.handle_update_users(&msg.session_id).await;

                if let Ok(res) = res {
                    ServerResult::Broadcast {
                        session_id: msg.session_id.to_string(),
                        msg: res,
                        exclude: None,
                    }
                    .distribute_message(&this)
                    .await;
                }
            })
        }
    }

    impl Handler<ClientMessageWrapper> for CharadeServer {
        type Result = ResponseFuture<()>;

        fn handle(&mut self, msg: ClientMessageWrapper, _: &mut Context<Self>) -> Self::Result {
            let this = self.clone();

            Box::pin(async move {
                let res = this
                    .handle_incoming_client_message(msg.message, &msg.id, &msg.session_id)
                    .await;

                match res {
                    Ok(res) => res.distribute_message(&this).await,
                    Err(res) => res.distribute_message(&this).await,
                }
            })
        }
    }
}
