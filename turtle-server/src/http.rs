use crate::http::routes::router;
use crate::turtles::{TurtleManager, TurtleNotification};
use kameo::actor::pubsub::PubSub;
use kameo::actor::ActorRef;
use sea_orm::DatabaseConnection;
use std::net::SocketAddr;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

mod routes;

pub async fn run(
    pub_sub: ActorRef<PubSub<TurtleNotification>>,
    manager: ActorRef<TurtleManager>,
    db: DatabaseConnection,
) {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let app = router(pub_sub, manager, db).layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::default().include_headers(false)),
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
