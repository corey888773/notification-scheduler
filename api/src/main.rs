use std::env;
use std::sync::Arc;
use axum::{Router};
use axum::routing::get;
use crate::app_state::AppState;

mod app_state;
mod api;
mod data;

#[tokio::main]
async fn main() {
    dotenvy::from_filename("api/src/app.env").ok();
    let port : String  = env::var("PORT").unwrap_or("8080".to_string());
    let db = data::db::DbContext::new(&env::var("MONGO_URL").unwrap()).await.unwrap();

    let app_state = AppState::new(db);
    let app = app(Arc::new(app_state));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind port");

    println!("Server running on port: {}", port);
    axum::serve(listener, app).await.expect("Server failed to start");
}


pub fn app(app_state: Arc<AppState>) -> Router {
    let api_routes = Router::new()
        .route("/hello", get(|| async { "Hello, World!" }))
        .merge(api::notifications::routes(app_state.clone()));

    Router::new()
        .nest("/api/v1", api_routes)
}