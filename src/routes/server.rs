use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use sqlx::PgPool;

use tower::ServiceBuilder;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tower_http::LatencyUnit;
use tracing::Level;

use crate::routes::{
    get_book,
    get_list_books,
    create_book,
    update_book,
    delete_book
};

/// Healthcheck GET
async fn hello() -> (StatusCode, String) {
    (StatusCode::OK, "hello".to_string())
}

/// Healthcheck POST
async fn hello_post() -> (StatusCode, String) {
    (StatusCode::OK, "hello post".to_string())
}

/// Parameters to hold in our State 
#[derive(Clone)]
pub struct ApiContext {
   pub db: PgPool,
}

/// Builds the base app
pub fn app(db: PgPool) -> Router {
    let api_context = ApiContext { db };
    return api_router(api_context);
}

/// Sets our API routes, our State, and our Trace Layer
fn api_router(api_context: ApiContext) -> Router {
    Router::new()
        .route("/api/healthcheck", get(hello).post(hello_post))
        .route("/api/books/list", get(get_list_books))
        .route(
            "/api/books",
            get(get_book)
                .post(create_book)
                .put(update_book)
                .delete(delete_book),
        )
        .with_state(api_context)
        .layer(
            ServiceBuilder::new().layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    )
                    .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
            ),
        )
}
