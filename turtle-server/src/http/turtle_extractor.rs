use crate::http::turtle_state::TurtleState;
use crate::turtles::{GetTurtle, Turtle, TurtleManager};
use axum::extract::{FromRef, FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::RequestPartsExt;
use kameo::actor::ActorRef;
use std::future::Future;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct TurtleManagerState(pub ActorRef<TurtleManager>);

impl FromRef<TurtleState> for TurtleManagerState {
    fn from_ref(input: &TurtleState) -> Self {
        TurtleManagerState(input.turtle_manager.clone())
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
            .extract::<Path<Arc<str>>>()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")))?;

        match manager
            .ask(GetTurtle {
                name: turtle_name.clone(),
            })
            .await
        {
            Ok(Some(turtle)) => Ok(Self(turtle)),
            Ok(None) => Err((
                StatusCode::NOT_FOUND,
                format!("Turtle {turtle_name} not connected"),
            )),
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "SendError trying to get turtle".to_owned(),
            )),
        }
    }
}
