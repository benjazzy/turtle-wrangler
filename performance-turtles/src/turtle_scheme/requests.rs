use serde::Serialize;
use crate::turtle_scheme::traits::{ Command, Request };
use crate::turtle_scheme::responses::Pong;

#[derive(Serialize)]
pub struct Ping;

impl Command for Ping {
    const NEEDS_LOCK: bool = false;
}

impl Request for Ping {
    type Response = Pong;
}