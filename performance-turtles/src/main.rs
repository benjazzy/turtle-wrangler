use actix::{Actor, Arbiter, Context, Handler, System};
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::server::{NewStream, TcpServer};

mod server;
mod turtle_manager;

struct Dummy;

impl Actor for Dummy {
    type Context = Context<Self>;
}

impl Handler<NewStream> for Dummy {
    type Result = ();

    fn handle(&mut self, msg: NewStream, ctx: &mut Self::Context) -> Self::Result {
        info!("Got it");
    }
}

pub fn main() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "turtle_wrangler=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let sys = System::new();

    let arbi = Arbiter::current();
    let server = TcpServer::start_in_arbiter(
        &arbi,
        |_| TcpServer::new("127.0.0.1:8080", Dummy.start().recipient())
    );

    sys.run().unwrap();
}
