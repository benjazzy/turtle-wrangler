use crate::notifications::{Note, Notification, NotificationRouter, Notify};
use crate::turtle::turtle_connection::{
    SendMessage, SetMessageHandler, TurtleConnection, WebsocketMessage,
};
use crate::turtle::Close;
use crate::turtle_manager::RegisterTurtle;
use crate::turtle_scheme::TurtleCommand;
use actix::prelude::*;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use super::turtle_receiver::TurtleReceiver;
use super::turtle_sender::{TurtleSender, TurtleSenderInner};
use super::Turtle;

pub struct TurtleIdentifier {
    unknown_turtles: HashMap<usize, Addr<TurtleConnection>>,
    next_id: usize,
    turtle_manager: Recipient<RegisterTurtle>,
    router: Addr<NotificationRouter>,
}

impl TurtleIdentifier {
    pub fn new(
        turtle_manager: Recipient<RegisterTurtle>,
        router: Addr<NotificationRouter>,
    ) -> Self {
        TurtleIdentifier {
            unknown_turtles: HashMap::new(),
            next_id: 0,
            turtle_manager,
            router,
        }
    }

    fn get_name(id: u64) -> &'static str {
        const NAMESLIST: NamesList = NamesList::new(include_str!("../../../first-names.txt"));

        if let Some(n) = NAMESLIST.get(id) {
            n
        } else {
            "Turtle"
        }
    }

    fn identify(message: WebsocketMessage) -> Result<&'static str, ()> {
        let turtle_id = match message {
            WebsocketMessage::Text(id) => id.trim().to_string(),
            WebsocketMessage::Close => {
                return Err(());
            }
        };

        let turtle_id: u64 = if let Ok(turtle_id) = turtle_id.parse::<f64>() {
            turtle_id as u64
        } else {
            error!("Turtle sent invalid id {turtle_id}");
            return Err(());
        };

        debug!("Turtle has id {turtle_id}");
        let name = Self::get_name(turtle_id);

        info!("{name} connected");

        Ok(name)
    }
}

impl Actor for TurtleIdentifier {
    type Context = Context<TurtleIdentifier>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewUnknownTurtle(pub Addr<TurtleConnection>);

impl Handler<NewUnknownTurtle> for TurtleIdentifier {
    type Result = ();

    fn handle(&mut self, msg: NewUnknownTurtle, ctx: &mut Self::Context) -> Self::Result {
        let turtle = msg.0;
        let addr = ctx.address();
        let identify_id = self.next_id;
        self.next_id += 1;

        let turtle_addr = turtle.clone();
        let turtle_manager = self.turtle_manager.clone();
        let router = self.router.clone();
        let result = turtle.try_send(SetMessageHandler(move |message: WebsocketMessage| {
            if let Ok(name) = Self::identify(message) {
                // Send the turtle its name.
                if turtle_addr.try_send(SendMessage(name.to_string())).is_err() {
                    warn!("Unable to send turtle {name} its name");
                    turtle_addr.do_send(Close);
                } else {
                    // Note when TurtleReceiver starts it registers its own message handler.
                    let sender_inner =
                        TurtleSenderInner::new(turtle_addr.clone(), name.to_string()).start();
                    let sender = TurtleSender::new(
                        turtle_addr.clone(),
                        sender_inner.clone(),
                        name.to_string(),
                    );
                    let receiver = TurtleReceiver::new(
                        name.clone(),
                        turtle_addr.clone(),
                        router.clone(),
                        sender_inner.clone(),
                    )
                    .start();
                    let known_turtle = Turtle::new(sender, receiver, name.to_string());
                    if let Err(err) = turtle_manager.try_send(RegisterTurtle(known_turtle)) {
                        error!(
                            "Unable to pass {name} on to the TurtleManager. Closing the connection"
                        );
                        err.into_inner().0.close();
                    }

                    if let Err(e) = router.try_send(Notify(Notification::Note(
                        Note::TurtleConnected(name.to_string()),
                    ))) {
                        error!("Problem sending turtle connected notification to router {e}");
                    }
                }
            } else {
                // If the turtle sent an invalid name close the connection.
                turtle_addr.do_send(Close);
            }

            addr.do_send(IdentifiedTurtle(identify_id));

            Ok(())
        }));

        if result.is_ok() {
            self.unknown_turtles.insert(identify_id, turtle);
        } else {
            warn!("Turtle closed before it could be identified");
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct IdentifiedTurtle(usize);

impl Handler<IdentifiedTurtle> for TurtleIdentifier {
    type Result = ();

    fn handle(&mut self, msg: IdentifiedTurtle, ctx: &mut Self::Context) -> Self::Result {
        if self.unknown_turtles.remove(&msg.0).is_none() {
            warn!("Turtle identified without being in list");
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
