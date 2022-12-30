use axum::Server;
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use library_api_rir::routes::app;

/// Spins up our app, sets debug level, gets the database started and binds
/// our server.
#[tokio::main]
async fn main() {
    println!("Server Spinning Up");

    for (n,v) in std::env::vars() {
        println!("{}: {}", n,v);
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "library_app_rir=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = option_env!("DATABASE_URL")
        .unwrap_or("$DATABASE_URL not set")
        .to_string();

    println!("db_url: {:?}", &database_url);

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("couldnt connect to database url");

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8081));
    Server::bind(&addr)
        .serve(app(db).into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    println!("Server Shutting Down");
}

/// Graceful Shutdown of the endpoint
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler")
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::interrupt())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
