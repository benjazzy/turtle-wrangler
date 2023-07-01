#[derive(Debug)]
pub enum TurtleSenderMessage {
    Close,
    Message(String),
}
