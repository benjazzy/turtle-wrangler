use std::sync::Arc;
use thiserror::Error;
use tracing::debug;
use turtle_types::turtle_scheme::{
    Coordinates, Heading, Position, ToolSide,
    turtle_messages::{
        Dig, DigDown, DigError, DigUp, Down, Forward, GetReport, MovementError, TurtleResult, Up,
    },
};

use crate::turtles::{
    TaskyTurtle, TurtleRequestError,
    turtle::task::{TaskName, TurtleTask, face_heading::FaceHeading},
};

macro_rules! tunnel {
    ($task_name:ident, $move:expr, $dig:tt) => {
        pub struct $task_name(pub ToolSide);

        impl TurtleTask for $task_name {
            type Return = Result<Position, NavigateToError>;

            type Fut<'a>
                = impl Future<Output = Self::Return>
            where
                Self: 'a;

            fn execute<'t, 'a>(self, turtle: &'t TaskyTurtle) -> Self::Fut<'a>
            where
                Self: 'a,
                't: 'a,
            {
                let $task_name(side) = self;
                async move {
                    loop {
                        match turtle.command($dig { side }).await? {
                            TurtleResult::Ok(_) | TurtleResult::Err(DigError::NothingToDig) => {
                                match turtle.command($move).await? {
                                    TurtleResult::Ok(pos) => return Ok(pos),
                                    TurtleResult::Err(MovementError::OutOfFuel) => {
                                        return Err(NavigateToError::OutOfFuel);
                                    }

                                    // Need to add a way for tasks to fail
                                    TurtleResult::Err(MovementError::MovementObstructed) => {
                                        continue;
                                    }
                                }
                            }
                            TurtleResult::Err(DigError::UnbreakableBlock) => {
                                return Err(NavigateToError::UnbreakableBlock);
                            }
                            TurtleResult::Err(DigError::NoTool) => {
                                return Err(NavigateToError::NoTool(NoToolError(side)));
                            }
                        }
                    }
                }
            }

            fn task_name(&self) -> impl Into<TaskName> {
                stringify!($task_name)
            }
        }
    };
}

#[derive(Debug, Error)]
#[error("No tool in {0}")]
pub struct NoToolError(pub ToolSide);

#[derive(Debug, Error)]
pub enum NavigateToError {
    #[error("Problem sending request to turtle {0}")]
    RequestError(#[from] TurtleRequestError),
    #[error("Unable to navigate because turtle ran out of fuel")]
    OutOfFuel,
    #[error("Unable to move because of unbreakable block")]
    UnbreakableBlock,
    #[error("{0}")]
    NoTool(NoToolError),
}

tunnel!(TunnelForward, Forward {}, Dig);
tunnel!(TunnelUp, Up {}, DigUp);
tunnel!(TunnelDown, Down {}, DigDown);

pub struct TunnelTo(pub Coordinates, pub ToolSide);

impl TurtleTask for TunnelTo {
    type Return = Result<bool, NavigateToError>;

    type Fut<'a>
        = impl Future<Output = Self::Return>
    where
        Self: 'a;

    fn execute<'t, 'a>(self, turtle: &'t TaskyTurtle) -> Self::Fut<'a>
    where
        Self: 'a,
        't: 'a,
    {
        let TunnelTo(target, tool_side) = self;
        async move {
            println!("NavigateTo");
            let mut position = turtle.command(GetReport {}).await?.coordinates;
            loop {
                let dif = dbg!(position - target);
                position = match dif {
                    Coordinates { x: 0, y: 0, z: 0 } => break,
                    Coordinates { x: 0, y, z: 0 } => {
                        if y > 0 {
                            turtle.run_task(TunnelDown(tool_side)).await?.coordinates
                        } else {
                            turtle.run_task(TunnelUp(tool_side)).await?.coordinates
                        }
                    }
                    Coordinates { x: 0, y: _, z } => {
                        if z > 0 {
                            turtle.run_task(FaceHeading(Heading::North)).await?;
                        } else {
                            turtle.run_task(FaceHeading(Heading::South)).await?;
                        }

                        turtle.run_task(TunnelForward(tool_side)).await?.coordinates
                    }
                    Coordinates { x, y: _, z: _ } => {
                        if x > 0 {
                            turtle.run_task(FaceHeading(Heading::West)).await?;
                        } else {
                            turtle.run_task(FaceHeading(Heading::East)).await?;
                        }

                        turtle.run_task(TunnelForward(tool_side)).await?.coordinates
                    }
                };
            }

            Ok(true)
        }
    }

    fn task_name(&self) -> impl Into<TaskName> {
        "naviage to"
    }
}
