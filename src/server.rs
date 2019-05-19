use actix::prelude::*;
use std::collections::HashMap;

/// Chat server sends this messages to session
#[derive(Message, Debug)]
pub struct OutboundMessage(pub String);

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
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
    seqno: usize,
}

impl ChatServer {
    fn broadcast_active_connections(&self) {
        for addr in self.sessions.values() {
            let _ = addr.try_send(OutboundMessage(format!("{} active", self.sessions.len())));
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

/// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> usize {
        // register session with unique id
        let id = self.seqno;
        self.seqno += 1;
        self.sessions.insert(id, msg.addr);

        // notify all users in same room
        self.broadcast_active_connections();

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // remove address
        self.sessions.remove(&msg.id);
        self.broadcast_active_connections();
    }
}
