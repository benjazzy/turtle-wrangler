use tokio::sync::mpsc;

use super::turtle_manager_message::TurtleManagerMessage;

pub struct TurtleManagerInner {
    rx: mpsc::Receiver<TurtleManagerMessage>,
}

impl TurtleManagerInner {
    pub fn new(rx: mpsc::Receiver<TurtleManagerMessage>) -> Self {
        TurtleManagerInner { rx }
    }

    // pub async fn run(mut self)
}
