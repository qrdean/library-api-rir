use axum::{Router,Json};
use axum::http::StatusCode;
use axum::routing::{get, post};

use tower::ServiceBuilder;
use tracing::Level;
use tower_http::LatencyUnit;
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, DefaultOnFailure};

use crate::routes::Result;

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

#[derive(serde::Serialize, serde::Deserialize)]
struct BookBody<T = BookQuery> {
    book: T,
}

#[derive(serde::Deserialize)]
struct BookQuery {
    lccn: String,
    isbn: Option<String>,
    title: String,
    author: String,
    publish_date: String,
}

#[derive(serde::Serialize, Default)]
struct BooksQuery {
    books: Vec<Book>
}

#[derive(serde::Deserialize)]
struct BookUpdateQuery {
    id: i32,
    lccn: Option<String>,
    isbn: Option<String>,
    title: Option<String>,
    author: Option<String>,
    publish_date: Option<String>,
}

#[derive(serde::Deserialize)]
struct DeleteBook {
    id: i32
}

async fn hello() -> (StatusCode, String) {
    (StatusCode::OK, "hello".to_string())
}

async fn hello_post() -> (StatusCode, String) {
    (StatusCode::OK, "hello post".to_string())
}

async fn get_list_books() -> (StatusCode, Json<BooksQuery>) {
    let mut books = BooksQuery::default();
    books.books.push(Book { id: 1, ..Default::default() });
    books.books.push(Book { id: 2, ..Default::default() });

    (StatusCode::OK, Json(books))
}

async fn get_book(request: Json<BookBody<DeleteBook>>) 
    -> Result<(StatusCode, Json<Book>), (StatusCode, String)> 
{
    let id = request.book.id;
    if id > 50 {
       return Err((StatusCode::NOT_FOUND, "Book Id not found".to_string()))
    }
    Ok((StatusCode::OK, Json(Book::default())))
}

async fn create_book(request: Json<BookBody<BookQuery>>) -> (StatusCode, String) {
    println!("{:?}", &request.book.lccn);
    println!("{:?}", &request.book.isbn);
    println!("{:?}", &request.book.title);
    println!("{:?}", &request.book.author);
    println!("{:?}", &request.book.publish_date);
    (StatusCode::OK, "Book Created".to_string())
}

async fn delete_book(request: Json<BookBody<DeleteBook>>) 
    -> Result<(StatusCode, String), (StatusCode, String)> 
{
    let id = request.book.id;
    // simulate id not being there
    if id > 50 {
        Err((StatusCode::NOT_FOUND, "Book Id Not Found".to_string()))
    } else {
        Ok((StatusCode::OK, "Book Deleted".to_string()))
    }
}

async fn update_book(request: Json<BookBody<BookUpdateQuery>>) 
    -> Result<(StatusCode, String), (StatusCode, String)> 
{
    let id = request.book.id;
    // simulate id not being there
    if id > 50 {
        Err((StatusCode::NOT_FOUND, "Book Id Not Found".to_string()))
    } else {
        println!("{:?}", &request.book.lccn);
        println!("{:?}", &request.book.isbn);
        println!("{:?}", &request.book.title);
        println!("{:?}", &request.book.author);
        println!("{:?}", &request.book.publish_date);
        Ok((StatusCode::OK, "Book Updated".to_string()))
    }
}

pub fn app() -> Router {
    Router::new()
        .route("/api/healthcheck", get(hello).post(hello_post))
        .route("/api/books/list", get(get_list_books))
        .route("/api/books", get(get_book).post(create_book).put(update_book).delete(delete_book))
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
