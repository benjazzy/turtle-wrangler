use crate::server::{NewStream, NewWebsocket, TcpServer, WebsocketAcceptor};
use actix::{Actor, Arbiter, Context, Handler, System};
use actix_web::{App, get, HttpRequest, HttpResponse, HttpServer, web};
use actix_web_actors::ws;
use tokio::net::TcpStream;
use tracing::{debug, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

mod server;
mod turtle;
mod turtle_manager;

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
async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    debug!("Got request");
    ws::start(turtle::turtle_connection::TurtleConnection::new(), &req, stream)
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "turtle_wrangler=trace".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    HttpServer::new(|| App::new().service(index))
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
