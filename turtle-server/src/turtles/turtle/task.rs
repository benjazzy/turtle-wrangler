// mod task_master;

use std::{future::Future, sync::Arc};

use crate::turtles::{
    turtle::{task_tracker::RegisterTask, TaskyTurtle},
    LockedTurtle, Queryable, Turtle,
};

pub trait TurtleTask {
    type Return;
    type Fut<'a>: Future<Output = Self::Return> + Send + 'a
    where
        Self: 'a;

    fn execute<'t, 'a>(self, turtle: &'t TaskyTurtle) -> Self::Fut<'a>
    where
        Self: 'a,
        't: 'a;
    fn task_name(&self) -> impl Into<Arc<str>>;
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

    fn task_name(&self) -> impl Into<Arc<str>> {
        "Generic fn"
    }
}
