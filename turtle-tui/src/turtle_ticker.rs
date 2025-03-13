use std::time::Duration;

use tokio::{
    sync::mpsc,
    time::{self},
};
use turtle_wrangler::scheme::{Turtle, TurtleReport};

use crate::app::AppMessage;

pub struct TurtleTicker {
    tx: mpsc::Sender<AppMessage>,
    client: reqwest::Client,
}

impl TurtleTicker {
    pub fn new(tx: mpsc::Sender<AppMessage>) -> Self {
        let client = reqwest::Client::new();

        TurtleTicker { tx, client }
    }

    pub fn start(self) {
        tokio::spawn(self.ticker_loop());
    }

    async fn ticker_loop(self) {
        let mut interval = time::interval(Duration::from_secs(1));
        while !self.tx.is_closed() {
            let turtles = self
                .client
                .get("http://localhost:8080/turtles")
                .send()
                .await
                .unwrap()
                .json::<Vec<TurtleReport>>()
                .await
                .unwrap();

            self.tx.send(AppMessage::Turtles(turtles)).await;

            interval.tick().await;
        }
    }
}
