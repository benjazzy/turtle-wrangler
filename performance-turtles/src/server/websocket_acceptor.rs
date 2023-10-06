use crate::server::NewStream;
use actix::prelude::*;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tracing::{error, warn};

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewWebsocket<S>(WebSocketStream<S>)
where
    S: AsyncRead + AsyncWrite + Unpin;

impl<S> NewWebsocket<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub fn into_inner(self) -> WebSocketStream<S> {
        self.0
    }
}
pub struct WebsocketAcceptor {
    recipient: Recipient<NewWebsocket<TcpStream>>,
}

impl WebsocketAcceptor {
    pub fn new(recipient: Recipient<NewWebsocket<TcpStream>>) -> Self {
        WebsocketAcceptor { recipient }
    }
}

impl Actor for WebsocketAcceptor {
    type Context = Context<Self>;
}

impl Handler<NewStream> for WebsocketAcceptor {
    type Result = ();
    fn handle(&mut self, msg: NewStream, ctx: &mut Self::Context) -> Self::Result {
        let recipient = self.recipient.clone();

        // Uses tokio_tungstenite to accept the ws stream and disconnects the stream if there is a
        // problem. Returns Err if self should close.
        let accept_fut = async move {
            let stream = match tokio_tungstenite::accept_async(msg.into_inner()).await {
                Ok(s) => s,
                Err(e) => {
                    error!("Problem accepting ws stream {e}");
                    return Ok(());
                }
            };

            if let Err(err) = recipient.try_send(NewWebsocket(stream)) {
                let (result, mut stream) = match err {
                    SendError::Closed(mut stream) => {
                        error!("Websocket recipient closed. Shutting down acceptor");

                        (Err(()), stream.0)
                    }
                    SendError::Full(mut stream) => {
                        warn!("Websocket recipient full");

                        (Ok(()), stream.0)
                    }
                };
                let _ = stream.close(None).await;

                return result;
            }

            Ok(())
        };

        // Turns the accept future into a actor future that can be run in context.
        let fut = fut::wrap_future(accept_fut);

        // Checks if the accept future returns an error. If it does then tell self to exit.
        let fut = fut.map(|result, _actor, ctx: &mut Context<Self>| {
            if result.is_err() {
                ctx.notify(AcceptCloseMessage);
            }
        });

        // Runs the future in our context.
        ctx.spawn(fut);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct AcceptCloseMessage;

impl Handler<AcceptCloseMessage> for WebsocketAcceptor {
    type Result = ();

    fn handle(&mut self, msg: AcceptCloseMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}
