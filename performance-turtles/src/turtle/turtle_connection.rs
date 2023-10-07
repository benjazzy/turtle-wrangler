use std::time::{Duration, Instant};
use actix::prelude::*;
use actix_web_actors::ws;
use actix_web_actors::ws::ProtocolError;
use tracing::{debug, warn};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct TurtleConnection {
    hb: Instant,
}

impl TurtleConnection {
    pub fn new() -> Self {
        TurtleConnection {
            hb: Instant::now(),
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {

        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                warn!("Websocket heartbeat timeout");

                ctx.stop();

                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for TurtleConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb = Instant::now();
        self.hb(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for TurtleConnection {
    fn handle(&mut self, msg: Result<ws::Message, ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Websocket error {e}");
                return;
            }
        };

        debug!("Websocket message: {:?}", msg);

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => self.hb = Instant::now(),
            ws::Message::Text(text) => {}
            ws::Message::Binary(_) => { warn!("Unexpected binary from turtle") }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => ctx.stop(),
            ws::Message::Nop => {}
        }
    }
}

