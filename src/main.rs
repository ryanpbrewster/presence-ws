use std::time::{Duration, Instant};

use actix::*;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

mod server;

use server::{ChatServer, Connect, Disconnect, OutboundMessage};
use std::sync::atomic::{AtomicUsize, Ordering};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Entry point for our route
fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<ChatServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        WsChatSession {
            id: CONNECTION_SEQNO.fetch_add(1, Ordering::Relaxed),
            hb: Instant::now(),
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

static CONNECTION_SEQNO: AtomicUsize = AtomicUsize::new(0);

struct WsChatSession {
    /// unique session id
    id: usize,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    /// Chat server
    addr: Addr<ChatServer>,
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register with the chat server
        self.addr.do_send(Connect {
            id: self.id,
            addr: ctx.address().recipient(),
        });
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        // optimistically notify chat server.  if this fails it's not a big
        // deal, the server will note that we're dead next time it talks to us.
        let _ = self.addr.try_send(Disconnect { id: self.id });
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<OutboundMessage> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: OutboundMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<ws::Message, ws::ProtocolError> for WsChatSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Text(_) => (),
            ws::Message::Binary(_) => (),
            ws::Message::Nop => (),
        }
    }
}

impl WsChatSession {
    /// send ping to client every second. also check heartbeats from client.
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                ctx.stop();
            } else {
                ctx.ping("");
            }
        });
    }
}

fn main() -> std::io::Result<()> {
    let sys = System::new("ws-example");

    // Start chat server actor
    let server = ChatServer::default().start();

    // Create Http server with websocket support
    HttpServer::new(move || {
        App::new()
            .data(server.clone())
            .service(web::resource("/ws/").to(chat_route))
    })
    .bind("[::]:8080")?
    .start();

    sys.run()
}
