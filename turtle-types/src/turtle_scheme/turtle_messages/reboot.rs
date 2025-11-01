use crate::turtle_scheme::turtle_messages::Command;
use serde::Serialize;

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "reboot")]
pub struct Reboot {
    pub id: u64,
}

impl Command for Reboot {
    type Response = u64;
}
