use std::sync::Arc;
use axum::extract::State;
use axum::{Json, Router};
use axum::response::IntoResponse;
use axum::routing::post;
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;

pub fn routes(state: Arc<AppState>) -> Router{
    let routes = Router::new()
        .route("/", post(test))
        .with_state(state);

    Router::new().nest("/notifs", routes)
}

#[derive(Debug, Deserialize, Serialize)]
struct TestRequest{
    #[serde(rename = "test")] test: String,
}
async fn test(state: State<Arc<AppState>>, req: Json<TestRequest>) -> impl IntoResponse{
    println!("->> {:<20}", "/notifs/test");

    Json("Hello, World!")

}