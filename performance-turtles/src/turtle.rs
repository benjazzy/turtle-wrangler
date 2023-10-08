pub mod turtle_connection;
pub mod turtle_identifier;
mod turtle_receiver;
mod turtle_sender;
mod unknown_turtle_connection;

use actix::prelude::*;

use self::turtle_receiver::TurtleReceiver;
use self::turtle_sender::TurtleSender;

// pub struct Turtle {
//     sender: TurtleSender,
//     receiver: Addr<TurtleReceiver>,
// }
//
// impl Turtle {
//     pub fn new(sender: TurtleSender, receiver: Addr<TurtleReceiver>) -> Self  {
//         Turtle { sender  receiver }
//     }
// }
