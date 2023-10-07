use crate::server::{NewStream, NewWebsocket, TcpServer, WebsocketAcceptor};
use crate::turtle::turtle_identifier::{NewUnknownTurtle, TurtleIdentifier};
use actix::{Actor, Addr, Arbiter, Context, Handler, System};
use actix_web::http::StatusCode;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use actix_web_actors::ws::WsResponseBuilder;
use tokio::net::TcpStream;
use tracing::{debug, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

mod scheme;
mod server;
mod turtle;
mod turtle_manager;
mod turtle_scheme;

struct Dummy;

impl Actor for Dummy {
    type Context = Context<Self>;
}

impl Handler<NewWebsocket<TcpStream>> for Dummy {
    type Result = ();

    fn handle(&mut self, msg: NewWebsocket<TcpStream>, ctx: &mut Self::Context) -> Self::Result {
        info!("Got it");
    }
}

#[get("/ws")]
async fn index(
    turtle_identifier: web::Data<Addr<TurtleIdentifier>>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    debug!("Got request");

    WsResponseBuilder::new(
        turtle::turtle_connection::TurtleConnection::new(),
        &req,
        stream,
    )
    .start_with_addr()
    .map(|(addr, response)| {
        let result = turtle_identifier.try_send(NewUnknownTurtle(addr));

        // If there is a problem registering the websocket then return 503.
        if result.is_ok() {
            response
        } else {
            HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
        }
    })
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "turtle_wrangler=trace".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let turtle_identifier = TurtleIdentifier::new().start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(turtle_identifier.clone()))
            .service(index)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

// pub fn main() {
//     tracing_subscriber::registry()
//         .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "turtle_wrangler=trace".into()))
//         .with(tracing_subscriber::fmt::layer())
//         .init();
//
//     let sys = System::new();
//
//     let arbi = Arbiter::current();
//     let server = TcpServer::start_in_arbiter(&arbi, |_| {
//         let ws_acceptor = WebsocketAcceptor::new(Dummy.start().recipient()).start();
//         TcpServer::new("127.0.0.1:8080", ws_acceptor.recipient())
//     });
//
//     sys.run().unwrap();
// }
