//use std::net::TcpListener;
use axum::{body::Body, http::{Request, StatusCode}};
//use tower::Service;
use tower::ServiceExt;
use library_app_rir::routes::app;

#[tokio::test]
async fn hello_world() {
    let app = app();

    // `Router` implements `tower::Service<Request<Body>>` so we can
    // call it like any tower service, no need to run an HTTP server.
    let response = app
        .oneshot(Request::builder()
        .method("GET")
        .uri("/api/healthcheck")
        .body(Body::empty())
        .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"hello");
}

#[tokio::test]
async fn hello_post() {
    let app = app();

    // `Router` implements `tower::Service<Request<Body>>` so we can
    // call it like any tower service, no need to run an HTTP server.
    let response = app
        .oneshot(Request::builder()
        .method("POST")
        .uri("/api/healthcheck")
        .body(Body::empty())
        .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"hello post");
}
