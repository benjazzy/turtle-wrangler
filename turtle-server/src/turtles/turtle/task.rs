mod face_heading;
mod strip_mine;
mod tunnel;

use std::{future::Future, sync::Arc};

use crate::turtles::{
    LockedTurtle, Queryable, Turtle,
    turtle::{TaskyTurtle, task_tracker::RegisterTask},
};

pub use face_heading::*;
use kameo::Reply;
pub use strip_mine::*;
pub use tunnel::*;

pub trait TurtleTask {
    type Return;
    type Fut<'a>: Future<Output = Self::Return> + Send + 'a
    where
        Self: 'a;

    fn execute<'t, 'a>(self, turtle: &'t TaskyTurtle) -> Self::Fut<'a>
    where
        Self: 'a,
        't: 'a;

    fn task_name(&self) -> impl Into<TaskName>;
}

impl<R, T> TurtleTask for T
where
    T: AsyncFnOnce(&TaskyTurtle) -> R,
    R: Send,
    for<'a> <T as AsyncFnOnce<(&'a TaskyTurtle,)>>::CallOnceFuture: Send,
{
    type Return = R;
    type Fut<'a>
        = <T as AsyncFnOnce<(&'a TaskyTurtle,)>>::CallOnceFuture
    where
        Self: 'a;

    fn execute<'t, 'a>(self, turtle: &'t TaskyTurtle) -> Self::Fut<'a>
    where
        T: 'a,
        't: 'a,
    {
        (self)(turtle)
    }

    fn task_name(&self) -> impl Into<TaskName> {
        "Generic fn"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reply)]
pub struct TaskName(pub &'static str);

impl AsRef<str> for TaskName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for TaskName {
    fn from(value: &'static str) -> Self {
        TaskName(value)
    }
}

impl std::fmt::Display for TaskName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
