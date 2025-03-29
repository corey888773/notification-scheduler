use std::sync::Arc;
use axum::extract::State;
use axum::{Json, Router};
use axum::response::IntoResponse;
use axum::routing::post;
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;

pub fn routes(state: Arc<AppState>) -> Router{
    let routes = Router::new()
        .route("/", post(create))
        .with_state(state);

    Router::new().nest("/notifications", routes)
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateRequest{
}
async fn create(state: State<Arc<AppState>>, req: Json<CreateRequest>) -> impl IntoResponse{
    println!("->> POST {:<20}", "/notifications");

    Json("Hello, World!")

}

async fn delete (state: State<Arc<AppState>>) -> impl IntoResponse{
    println!("->> DELETE {:<20}", "/notifications/:id");

    Json("Hello, World!")
}