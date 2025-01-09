use crate::turtles::turtle::{
    Turtle, TurtleNote, TurtleNotification, TurtleReceiver, TurtleSender,
};
use axum::extract::ws;
use futures::StreamExt;
use kameo::actor::pubsub::{PubSub, Publish};
use kameo::actor::ActorRef;
use kameo::message::{Context, Message};
use kameo::Actor;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

pub async fn identify_turtle(
    mut connection: ws::WebSocket,
    pub_sub: ActorRef<PubSub<TurtleNotification>>,
) {
    const NAMES: NamesList = NamesList::new(include_str!("../../first-names.txt"));

    match tokio::time::timeout(Duration::from_secs(100), connection.recv()).await {
        Ok(Some(Ok(ws::Message::Text(msg)))) => {
            let id = match u64::from_str(msg.as_str()) {
                Ok(id) => id,
                Err(e) => {
                    warn!("Problem parsing turtle id {e}");
                    return;
                }
            };

            let name = NAMES.get(id).unwrap_or("Turtle");
            info!("Turtle connection from {name}");
            connection.send(ws::Message::Text(name.into())).await;
            let name: Arc<str> = name.into();

            let (sink, stream) = connection.split();
            let sender = kameo::spawn(TurtleSender::new(name.clone(), sink));
            let receiver = kameo::actor::spawn_with(|actor_ref| async {TurtleReceiver::new(
                actor_ref,
                name.clone(),
                sender.clone(),
                stream,
                pub_sub.clone(),
            )}).await;
            let turtle = Turtle::new(name, sender, receiver);
            pub_sub
                .tell(Publish(TurtleNotification::Note(
                    TurtleNote::TurtleConnected(turtle),
                )))
                .await;
        }
        Ok(Some(Ok(_))) => warn!("Turtle sent invalid identification message"),
        Ok(Some(Err(e))) => warn!("Error identifying turtle {e}"),
        Ok(None) => warn!("Turtle sent empty message when identifying"),
        Err(timeout) => warn!("Turtle identification timed out: {timeout:?}"),
    }
}

#[derive(Actor)]
pub struct TurtleIdentifier {
    unknown_turtles: HashMap<usize, ws::WebSocket>,
    next_id: usize,
}

impl TurtleIdentifier {
    pub fn new() -> TurtleIdentifier {
        TurtleIdentifier {
            unknown_turtles: HashMap::new(),
            next_id: 0,
        }
    }

    fn get_name(id: u64) -> &'static str {
        const NAMES: NamesList = NamesList::new(include_str!("../../first-names.txt"));

        NAMES.get(id).unwrap_or("Turtle")
    }
}

pub struct UnknownTurtle(pub ws::WebSocket);

impl Message<UnknownTurtle> for TurtleIdentifier {
    type Reply = ();

    async fn handle(
        &mut self,
        UnknownTurtle(mut connection): UnknownTurtle,
        ctx: Context<'_, Self, Self::Reply>,
    ) {
        let id = self.next_id;
        self.next_id += 1;
        let name = Self::get_name(id as u64);

        if let Err(e) = connection.send(ws::Message::Text(name.into())).await {
            warn!("Unable to send {name} its name");
            return;
        }
    }
}

struct NamesList(&'static str);

impl NamesList {
    pub const fn new(names: &'static str) -> Self {
        NamesList(names)
    }

    pub fn get(&self, id: u64) -> Option<&'static str> {
        let name = self.0.split_whitespace().nth(id as usize)?;

        Some(name)
    }
}
