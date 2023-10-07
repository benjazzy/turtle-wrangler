use actix::prelude::*;
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub enum WebsocketMessage {
    Text(String),
    Close,
}

enum HandlerCaller {
    Set(Box<dyn Fn(WebsocketMessage)>),
    Unset(Vec<WebsocketMessage>),
}

impl HandlerCaller {
    pub fn new() -> Self {
        HandlerCaller::Unset(Vec::new())
    }

    pub fn set(&mut self, handler: impl Fn(WebsocketMessage) + 'static) {
        let new = HandlerCaller::Set(Box::new(handler));
        let old = std::mem::replace(self, new);

        if let HandlerCaller::Unset(messages) = old {
            for message in messages {
                self.handle(message);
            }
        }
    }

    pub fn handle(&mut self, message: WebsocketMessage) {
        match self {
            HandlerCaller::Set(handler) => handler(message),
            HandlerCaller::Unset(messages) => messages.push(message),
        }
    }

    pub fn are_unhandled(&self) -> bool {
        match self {
            HandlerCaller::Set(_) => false,
            HandlerCaller::Unset(messages) => !messages.is_empty(),
        }
    }

    pub fn flush(&mut self) {
        if let HandlerCaller::Unset(messages) = self {
            messages.clear();
        }
    }
}

pub struct TurtleConnection {
    hb: Instant,
    message_handler: HandlerCaller,
}

impl TurtleConnection {
    pub fn new() -> Self {
        TurtleConnection {
            hb: Instant::now(),
            message_handler: HandlerCaller::new(),
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

    /// Checks if there are still messages that have not been handled.
    /// If there are then continue running.
    /// If in 5 seconds there are still unhandled messages then clear the man stop.
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        if self.message_handler.are_unhandled() {
            ctx.run_later(Duration::from_secs(5), |act, ctx| {
                act.message_handler.flush();
                ctx.stop();
            });
            return Running::Continue;
        }

        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for TurtleConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
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
            ws::Message::Text(text) => {
                self.message_handler
                    .handle(WebsocketMessage::Text(text.to_string()));
            }
            ws::Message::Binary(_) => {
                warn!("Unexpected binary from turtle")
            }
            ws::Message::Close(reason) => {
                self.message_handler.handle(WebsocketMessage::Close);
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => ctx.stop(),
            ws::Message::Nop => {}
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage(pub String);

impl Handler<SendMessage> for TurtleConnection {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetMessageHandler<F: Fn(WebsocketMessage)>(pub F);

impl<F: Fn(WebsocketMessage) + 'static> Handler<SetMessageHandler<F>> for TurtleConnection {
    type Result = ();

    fn handle(&mut self, msg: SetMessageHandler<F>, _ctx: &mut Self::Context) -> Self::Result {
        self.message_handler.set(msg.0);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CloseMessage;

impl Handler<CloseMessage> for TurtleConnection {
    type Result = ();

    fn handle(&mut self, _: CloseMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.close(None);
    }
}
