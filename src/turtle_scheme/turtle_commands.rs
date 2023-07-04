use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TurtleCommands {
    Forward,
    Back,
    TurnLeft,
    TurnRight,
    Reboot,
    Inspect,
}
