use axum::{Router,Json};
use axum::http::StatusCode;
use axum::routing::get;

use tower::ServiceBuilder;
use tracing::Level;
use tower_http::LatencyUnit;
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, DefaultOnFailure};

#[derive(serde::Serialize, Default)]
struct Book {
    id: i32,
    master_id: i32,
    location_id: i32,
    author: String,
    title: String,
    lccn: String,
    isbn: String,
    publish_date: String,
} 

async fn hello() -> (StatusCode, String) {
    (StatusCode::OK, "hello".to_string())
}

async fn hello_post() -> (StatusCode, String) {
    (StatusCode::OK, "hello post".to_string())
}

async fn get_all_books() -> (StatusCode, Json<Book>){
    (StatusCode::OK, Json(Book::default()))
}

pub fn app() -> Router {
    Router::new()
        .route("/api/healthcheck", get(hello).post(hello_post))
        .route("/api/books/all", get(get_all_books))
        .layer(ServiceBuilder::new()
            .layer(TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new().include_headers(true)
                )
                .on_request(
                    DefaultOnRequest::new().level(Level::INFO)
                )
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros)
                )
                .on_failure(
                    DefaultOnFailure::new().level(Level::ERROR)
                )
            ),
        )
}
