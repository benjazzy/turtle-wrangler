use thiserror::Error;
use turtle_types::turtle_scheme::{
    Coordinates, Heading, Position, ToolSide,
    turtle_messages::{
        Dig, DigDown, DigError, DigUp, Down, Forward, GetReport, MovementError, TurtleResult, Up,
    },
};

use crate::turtles::{
    TaskyTurtle, TurtleRequestError,
    turtle::task::{TurtleTask, face_heading::FaceHeading},
};

macro_rules! tunnel {
    ($func_name:ident, $move:expr, $dig:tt) => {
        fn $func_name(
            side: ToolSide,
        ) -> impl TurtleTask<Return = Result<Position, NavigateToError>> {
            async move |turtle: &TaskyTurtle| {
                for _ in 0..5 {
                    match turtle
                    .command($dig {
                        side,
                    })
                    .await?
                    {
                        TurtleResult::Ok(_) | TurtleResult::Err(DigError::NothingToDig) => {
                            match turtle.command($move).await? {
                                TurtleResult::Ok(pos) => return Ok(pos),
                                TurtleResult::Err(MovementError::OutOfFuel) => return Err(NavigateToError::OutOfFuel),

                                // Need to add a way for tasks to fail
                                TurtleResult::Err(MovementError::MovementObstructed) => {
                                    panic!(
                                        "While attempting to navigate {} ran into a block that should have already been mined",
                                        turtle.get_name()
                                    );
                                }
                            }
                        }
                        TurtleResult::Err(DigError::UnbreakableBlock) => return Err(NavigateToError::UnbreakableBlock),
                        TurtleResult::Err(DigError::NoTool) => return Err(NavigateToError::NoTool(NoToolError(side))),
                    }
                }
                todo!()
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

tunnel!(tunnel_forward, Forward {}, Dig);
tunnel!(tunnel_up, Up {}, DigUp);
tunnel!(tunnel_down, Down {}, DigDown);

pub fn navigate_to(
    target: Coordinates,
    tool_side: ToolSide,
) -> impl TurtleTask<Return = Result<bool, NavigateToError>> {
    async move |turtle: &TaskyTurtle| {
        let mut position = turtle.command(GetReport).await?.position;
        loop {
            let dif = position - target;
            position = match dif {
                Coordinates { x: 0, y: 0, z: 0 } => break,
                Coordinates { x: 0, y, z: 0 } => {
                    if y > 0 {
                        turtle.run_task(tunnel_down(tool_side)).await?.coordinates
                    } else {
                        turtle.run_task(tunnel_up(tool_side)).await?.coordinates
                    }
                }
                Coordinates { x: 0, y: _, z } => {
                    if z > 0 {
                        turtle.run_task(FaceHeading(Heading::North)).await?;
                    } else {
                        turtle.run_task(FaceHeading(Heading::South)).await?;
                    }

                    turtle
                        .run_task(tunnel_forward(tool_side))
                        .await?
                        .coordinates
                }
                Coordinates { x, y: _, z: _ } => {
                    if x > 0 {
                        turtle.run_task(FaceHeading(Heading::West)).await?;
                    } else {
                        turtle.run_task(FaceHeading(Heading::East)).await?;
                    }

                    turtle
                        .run_task(tunnel_forward(tool_side))
                        .await?
                        .coordinates
                }
            };
        }

        Ok(true)
    }
}
