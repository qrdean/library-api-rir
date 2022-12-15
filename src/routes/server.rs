use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use sqlx::types::Uuid;
use sqlx::PgPool;
//use uuid::Uuid;

use tower::ServiceBuilder;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tower_http::LatencyUnit;
use tracing::Level;

use crate::db::Db;
use crate::routes::ApiError;
use crate::routes::Result;

#[derive(serde::Serialize, serde::Deserialize)]
struct BookBody<T = BookQuery> {
    book: T,
}

#[derive(serde::Serialize, Default)]
pub struct Book {
    id: Uuid,
    master_id: Option<Uuid>,
    location_id: Option<Uuid>,
    author: Option<String>,
    title: Option<String>,
    lccn: Option<String>,
    isbn: Option<String>,
    publish_date: Option<String>,
}

#[derive(Clone)]
pub struct BookFromQuery {
    pub master_book_id: Uuid,
    pub author: Option<String>,
    pub title: Option<String>,
    pub lccn: Option<String>,
    pub isbn: Option<String>,
    pub publish_date: Option<String>,
}

impl BookFromQuery {
    fn to_book(&self) -> Book {
        let this = self.to_owned();
        Book {
            id: self.master_book_id,
            master_id: None,
            location_id: None,
            author: this.author,
            title: this.title,
            lccn: this.lccn,
            isbn: this.isbn,
            publish_date: this.publish_date,
        }
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct BookQuery {
    pub lccn: String,
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub publish_date: String,
}

#[derive(serde::Serialize, Default)]
struct BooksQuery {
    books: Vec<Book>,
}

#[derive(serde::Deserialize, Clone)]
pub struct BookUpdateQuery {
    pub id: Uuid,
    pub lccn: Option<String>,
    pub isbn: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub publish_date: Option<String>,
}

#[derive(serde::Deserialize)]
struct GetBookQuery {
    id: Uuid,
}

#[derive(serde::Deserialize)]
struct DeleteBook {
    id: Uuid,
}

async fn hello() -> (StatusCode, String) {
    (StatusCode::OK, "hello".to_string())
}

async fn hello_post() -> (StatusCode, String) {
    (StatusCode::OK, "hello post".to_string())
}

async fn get_list_books(
    State(api_context): State<ApiContext>,
) -> Result<(StatusCode, Json<BooksQuery>), ApiError> {
    let connection_pool = &api_context.db;

    let list = Db::get_book_list(connection_pool).await?;
    let books = list.iter().map(|book| book.to_book()).collect();

    Ok((StatusCode::OK, Json(BooksQuery { books })))
}

async fn get_book(
    State(api_context): State<ApiContext>,
    request: Json<BookBody<GetBookQuery>>,
) -> Result<(StatusCode, Json<Book>), ApiError> {
    let connection_pool = &api_context.db;
    let id = &request.book.id;

    let book_from_query = Db::get_book(id, connection_pool).await?;

    Ok((StatusCode::OK, Json(book_from_query.to_book())))
}

async fn create_book(
    State(api_context): State<ApiContext>,
    request: Json<BookBody<BookQuery>>,
) -> Result<(StatusCode, String), ApiError> {
    let connection_pool = &api_context.db;
    let book = request.book.to_owned();
    let book_query = BookQuery {
        isbn: book.isbn,
        lccn: book.lccn,
        title: book.title,
        author: book.author,
        publish_date: book.publish_date,
    };
    let book_id = Db::create_book(book_query, connection_pool).await?;

    Ok((StatusCode::OK, book_id.to_string()))
}

async fn update_book(
    State(api_context): State<ApiContext>,
    Json(request): Json<BookBody<BookUpdateQuery>>,
) -> Result<(StatusCode, Json<Book>), ApiError> {
    let connection_pool = &api_context.db;
    let book_body = request.book.to_owned();
    let book_query = BookUpdateQuery {
        id: book_body.id,
        isbn: book_body.isbn,
        lccn: book_body.lccn,
        title: book_body.title,
        author: book_body.author,
        publish_date: book_body.publish_date,
    };

    let updated_book = Db::update_book(book_query, connection_pool).await?;

    let book = updated_book.to_book();

    Ok((StatusCode::OK, Json(book)))
}

async fn delete_book(
    State(api_context): State<ApiContext>,
    request: Json<BookBody<DeleteBook>>,
) -> Result<(StatusCode, String), ApiError> {
    let connection_pool = &api_context.db;

    let result = Db::delete_book(request.book.id, connection_pool).await?;

    Ok((StatusCode::OK, result.rows_affected().to_string()))
}

#[derive(Clone)]
struct ApiContext {
    db: PgPool,
}

pub fn app(db: PgPool) -> Router {
    let api_context = ApiContext { db };
    return api_router(api_context);
}

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
