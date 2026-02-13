use crate::turtles::turtle::{
    Turtle, TurtleNote, TurtleNotification, TurtleReceiver, TurtleSender,
};
use axum::extract::ws;
use futures::StreamExt;
use kameo::Actor;
use kameo::actor::{ActorRef, Spawn};
use kameo::message::{Context, Message};
use kameo_actors::pubsub::{PubSub, Publish};
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, sea_query};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use turtle_types::turtle_scheme::{Heading, TurtleType};

pub async fn identify_turtle(
    mut connection: ws::WebSocket,
    pub_sub: ActorRef<PubSub<TurtleNotification>>,
    db: DatabaseConnection,
) {
    const NAMES: NamesList = NamesList::new(include_str!("../../../first-names.txt"));

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

            let active_model = turtle_entities::turtle::ActiveModel {
                id: ActiveValue::set(id as i32),
                name: ActiveValue::set(name.as_ref().to_owned()),
                turtle_type: ActiveValue::set(TurtleType::Normal),
                fuel: ActiveValue::set(0),
                x: ActiveValue::set(0),
                y: ActiveValue::set(0),
                z: ActiveValue::set(0),
                heading: ActiveValue::set(Heading::North),
                inventory: ActiveValue::set(Default::default()),
                last_seen: ActiveValue::set(chrono::Utc::now()),
            };

            if let Err(e) = turtle_entities::turtle::Entity::insert(active_model)
                .on_conflict(
                    sea_query::OnConflict::new()
                        .update_column(turtle_entities::turtle::Column::Name)
                        .value(
                            turtle_entities::turtle::Column::Name,
                            name.as_ref().to_owned(),
                        )
                        .to_owned(),
                )
                .exec(&db)
                .await
            {
                error!("Problem updating database with new turtle connection {e}");
            }

            let (sink, stream) = connection.split();
            let sender = TurtleSender::spawn(TurtleSender::new(name.clone(), sink));
            let receiver = TurtleReceiver::spawn(TurtleReceiver::new(
                id,
                name.clone(),
                sender.clone(),
                stream,
                pub_sub.clone(),
                db.clone(),
            ));
            let turtle = Turtle::new(name, sender, receiver, pub_sub.clone());
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
        const NAMES: NamesList = NamesList::new(include_str!("../../../first-names.txt"));

        NAMES.get(id).unwrap_or("Turtle")
    }
}

pub struct UnknownTurtle(pub ws::WebSocket);

impl Message<UnknownTurtle> for TurtleIdentifier {
    type Reply = ();

    async fn handle(
        &mut self,
        UnknownTurtle(mut connection): UnknownTurtle,
        ctx: &mut Context<Self, Self::Reply>,
    ) {
        let id = self.next_id;
        self.next_id += 1;
        let name = Self::get_name(id as u64);

        if let Err(e) = connection.send(ws::Message::Text(name.into())).await {
            warn!("Unable to send {name} its name");
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
