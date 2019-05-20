use actix::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

/// Chat server sends this messages to session
#[derive(Message, Debug)]
pub struct OutboundMessage(pub String);

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
pub struct Connect {
    pub id: usize,
    pub addr: Recipient<OutboundMessage>,
}

/// Session is disconnected
#[derive(Message)]
pub struct Disconnect {
    pub id: usize,
}

/// `ChatServer` tracks active connections and broadcasts connection counts.
#[derive(Default)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<OutboundMessage>>,
}

impl ChatServer {
    fn broadcast_active_connections(&mut self) {
        let msg = format!("{} active", self.sessions.len());
        self.sessions
            .retain(|_, addr| addr.try_send(OutboundMessage(msg.clone())).is_ok());
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_millis(5000), |actor, _| {
            actor.broadcast_active_connections();
        });
    }
}

/// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        // register session with unique id
        self.sessions.insert(msg.id, msg.addr);
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // remove address
        self.sessions.remove(&msg.id);
    }
}
