//use std::net::TcpListener;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
//use tower::Service;
use library_app_rir::routes::app;
use tower::ServiceExt;

#[tokio::test]
async fn hello_world() {
    let database_url = "postgresql://postgres:password@localhost:5432/library".to_string();

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("couldnt connect to database url");

    let app = app(db);

    // `Router` implements `tower::Service<Request<Body>>` so we can
    // call it like any tower service, no need to run an HTTP server.
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/healthcheck")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"hello");
}

#[tokio::test]
async fn hello_post() {
    let database_url = "postgresql://postgres:password@localhost:5432/library";

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("couldnt connect to database url");

    let app = app(db);

    // `Router` implements `tower::Service<Request<Body>>` so we can
    // call it like any tower service, no need to run an HTTP server.
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/healthcheck")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"hello post");
}

