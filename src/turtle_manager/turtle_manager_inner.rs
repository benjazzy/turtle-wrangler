use tokio::sync::mpsc;
use tracing::{error, info};

use super::{
    turtle_manager_message::TurtleManagerMessage, turtle_status::TurtleStatus,
    unknown_turtle_connection::UnknownTurtleConnection, TurtleManagerHandle,
};

pub struct TurtleManagerInner {
    rx: mpsc::Receiver<TurtleManagerMessage>,
    own_handle: TurtleManagerHandle,
    turtles: Vec<TurtleStatus>,
}

impl TurtleManagerInner {
    pub fn new(rx: mpsc::Receiver<TurtleManagerMessage>, own_handle: TurtleManagerHandle) -> Self {
        TurtleManagerInner {
            rx,
            own_handle,
            turtles: Vec::new(),
        }
    }

    pub async fn run(mut self) {
        let mut close_tx = None;

        while let Some(message) = self.rx.recv().await {
            match message {
                TurtleManagerMessage::Close(tx) => {
                    close_tx = Some(tx);
                    break;
                }
                TurtleManagerMessage::UnknownTurtle(unknown_turtle) => {
                    self.new_unknown_turtle(unknown_turtle).await;
                }
                TurtleManagerMessage::Disconnnect(name) => self.disconnnect_turtle(name).await,
                TurtleManagerMessage::Broadcast(message) => self.broadcast(message).await,
                TurtleManagerMessage::Status(tx) => {
                    let names: String = self.turtles.iter().map(|t| format!("{t}\n")).collect();
                    let names = names.trim().to_string();

                    if tx.send(names).is_err() {
                        error!("Problem sending status");
                    };
                }
            }
        }

        info!("Turtle manager closing");
        if let Some(tx) = close_tx {
            let _ = tx.send(());
        }
    }

    async fn new_unknown_turtle(&mut self, unknown_turtle: UnknownTurtleConnection) {
        if let Some((name, connection)) = unknown_turtle.auth(self.own_handle.clone()).await {
            for turtle in self.turtles.iter_mut() {
                if turtle.get_name() == name {
                    if let Err(e) = turtle.connect(connection) {
                        error!("Problem authing turtle {e}");
                        let _ = turtle.disconnect().await;
                    }
                    return;
                }
            }

            self.turtles
                .push(TurtleStatus::Connected { name, connection });
        };
    }

    async fn disconnnect_turtle(&mut self, name: String) {
        for turtle in self.turtles.iter_mut() {
            if turtle.get_name() == name {
                if let Err(e) = turtle.disconnect().await {
                    error!("Problem disconnecting turtle {e}");
                }
                return;
            }
        }

        error!("Turtle named {name} attempted to disconnect without authing");
    }

    async fn broadcast(&mut self, message: String) {
        for turtle in self.turtles.iter() {
            if let TurtleStatus::Connected { connection, .. } = turtle {
                connection.send(message.clone()).await;
            }
        }
    }
}
