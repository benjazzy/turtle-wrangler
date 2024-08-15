use crate::client_manager::client_receiver_message::ClientReceiverMessage;
use tokio::net::tcp::ReadHalf;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::debug;

// pub struct ClientReceiverInner {
//     rx: mpsc::Receiver<ClientReceiverMessage>,
//     receiver: ReadHalf,
// }
//
// impl ClientReceiverInner {
//     pub fn new(rx: mpsc::Receiver<ClientReceiverMessage>, receiver: ReadHalf<TcpStream>) -> Self {
//         ClientReceiverInner { rx, receiver }
//     }
//
//     pub async fn run(mut self) {
//         let close_tx = None;
//
//         loop {
//             let mut buffer = [0; 1024];
//
//             tokio::select! {
//                 message = self.rx.recv() => {
//                     match message {
//                         ClientReceiverMessage::Close(tx) => {
//                             close_tx = Some(tx);
//                             break;
//                         }
//                     }
//                 }
//                 read_result = self.receiver.read(buffer) => {
//
//                 }
//             }
//         }
//
//         debug!("Client receiver closing");
//     }
// }
