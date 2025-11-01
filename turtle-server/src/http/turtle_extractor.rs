use crate::turtles::{GetTurtle, Turtle, TurtleManager};
use axum::extract::{FromRef, FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::RequestPartsExt;
use kameo::actor::ActorRef;
use std::future::Future;

#[derive(Debug, Clone)]
pub struct TurtleManagerState(pub ActorRef<TurtleManager>);

impl FromRef<ActorRef<TurtleManager>> for TurtleManagerState {
    fn from_ref(input: &ActorRef<TurtleManager>) -> Self {
        TurtleManagerState(input.clone())
    }
}

pub struct TurtleExtractor(pub Turtle);

impl<S> FromRequestParts<S> for TurtleExtractor
where
    S: Send + Sync,
    TurtleManagerState: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TurtleManagerState(manager) = TurtleManagerState::from_ref(state);
        let Path(turtle_name) = parts
            .extract::<Path<String>>()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")))?;

        match manager
            .ask(GetTurtle {
                name: turtle_name.clone().into(),
            })
            .await
        {
            Ok(Some(turtle)) => Ok(Self(turtle)),
            Ok(None) => Err((
                StatusCode::NOT_FOUND,
                format!("Turtle {turtle_name} not found"),
            )),
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "SendError trying to get turtle".to_owned(),
            )),
        }
    }
}
