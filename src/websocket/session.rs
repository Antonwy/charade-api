use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;

use crate::{
    repositories::database::Database,
    websocket::messages::{self, ClientMessage, ClientMessageWrapper},
};

use super::server;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsCharadeSession {
    /// user id
    pub id: String,

    /// session id
    pub session_id: String,

    /// last heart beat time
    pub hb: Instant,

    /// Chat server
    pub server: Addr<server::CharadeServer>,

    pub db: Database,
}

impl WsCharadeSession {
    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                act.server.do_send(messages::Disconnect {
                    id: act.id.clone(),
                    session_id: act.session_id.clone(),
                });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WsCharadeSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        self.server
            .send(messages::Connect {
                addr,
                id: self.id.clone(),
                session_id: self.session_id.clone(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.server.do_send(messages::Disconnect {
            id: self.id.clone(),
            session_id: self.session_id.clone(),
        });
        Running::Stop
    }
}

impl Handler<messages::ServerMessage> for WsCharadeSession {
    type Result = ();

    fn handle(&mut self, msg: messages::ServerMessage, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

impl Handler<messages::Message> for WsCharadeSession {
    type Result = ();

    fn handle(&mut self, msg: messages::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsCharadeSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let message_result = serde_json::from_str::<ClientMessage>(&text);

                match message_result {
                    Ok(msg) => {
                        self.server.do_send(ClientMessageWrapper {
                            id: self.id.clone(),
                            session_id: self.session_id.clone(),
                            message: msg,
                        });
                    }
                    Err(_) => {
                        println!("Not a json message");
                    }
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
