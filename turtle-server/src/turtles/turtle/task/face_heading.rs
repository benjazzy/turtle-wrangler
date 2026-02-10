use std::cmp::Ordering;

use sea_orm::Order;
use turtle_types::turtle_scheme::{
    Heading,
    turtle_messages::{GetReport, TurnLeft, TurnRight, TurtleResult},
};

use crate::turtles::{
    TaskyTurtle, TurtleRequestError,
    turtle::task::{TaskName, TurtleTask},
};

pub struct FaceHeading(pub Heading);

impl TurtleTask for FaceHeading {
    type Return = Result<(), TurtleRequestError>;

    type Fut<'a>
        = impl Future<Output = Self::Return>
    where
        Self: 'a;

    fn execute<'t, 'a>(self, turtle: &'t TaskyTurtle) -> Self::Fut<'a>
    where
        Self: 'a,
        't: 'a,
    {
        enum TurnDirection {
            Normal,
            Reversed,
        }

        fn turn_direction(dif: i8) -> TurnDirection {
            if dif.abs() < 2 {
                TurnDirection::Normal
            } else {
                TurnDirection::Reversed
            }
        }

        fn h_val(h: Heading) -> i8 {
            match h {
                Heading::North => 0,
                Heading::East => 1,
                Heading::South => 2,
                Heading::West => 3,
            }
        }

        let FaceHeading(target_heading) = self;
        async move {
            let mut heading = turtle.command(GetReport {}).await?.heading;
            loop {
                let dif = h_val(heading) - h_val(target_heading);

                // TODO: Check if turning can fail
                heading = match (dif.cmp(&0), turn_direction(dif)) {
                    // Right
                    (Ordering::Less, TurnDirection::Normal)
                    | (Ordering::Greater, TurnDirection::Reversed) => {
                        let TurtleResult::Ok(pos) = turtle.command(TurnRight {}).await? else {
                            panic!("Unable to rotate turtle TODO");
                        };

                        pos.heading
                    }

                    // Left
                    (Ordering::Greater, TurnDirection::Normal)
                    | (Ordering::Less, TurnDirection::Reversed) => {
                        let TurtleResult::Ok(pos) = turtle.command(TurnRight {}).await? else {
                            panic!("Unable to rotate turtle TODO");
                        };

                        pos.heading
                    }
                    (Ordering::Equal, _) => return Ok(()),
                };
            }
        }
    }

    fn task_name(&self) -> impl Into<TaskName> {
        "face heading"
    }
}
