use crate::http::turtle_state::TurtleState;
use axum::extract::{FromRef, FromRequestParts, Path};
use axum::http::request::Parts;
use axum::RequestPartsExt;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::sync::Arc;
use tokio_tungstenite::tungstenite::http::StatusCode;
use turtle_entities::turtle::Model as TurtleModel;

#[derive(Debug, Clone)]
pub struct TurtleDatabase(pub DatabaseConnection);

impl FromRef<TurtleState> for TurtleDatabase {
    fn from_ref(input: &TurtleState) -> Self {
        Self(input.database.clone())
    }
}

pub struct TurtleEntityExtractor(pub TurtleModel);

impl<S> FromRequestParts<S> for TurtleEntityExtractor
where
    S: Send + Sync,
    TurtleDatabase: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TurtleDatabase(db) = TurtleDatabase::from_ref(state);
        let Path(turtle_name) = parts
            .extract::<Path<Arc<str>>>()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")))?;

        match turtle_entities::turtle::Entity::find()
            .filter(turtle_entities::turtle::Column::Name.eq(turtle_name.as_ref()))
            .one(&db)
            .await
        {
            Ok(Some(turtle)) => Ok(Self(turtle)),
            Ok(None) => Err((
                StatusCode::NOT_FOUND,
                format!("Turtle {turtle_name} not in database"),
            )),
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
        }
    }
}
