use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use axum::extract::State;
use sqlx::PgPool;
use uuid::Uuid;

use tower::ServiceBuilder;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tower_http::LatencyUnit;
use tracing::Level;

use crate::routes::Result;

#[derive(serde::Serialize, serde::Deserialize)]
struct BookBody<T = BookQuery> {
    book: T,
}

#[derive(serde::Serialize, Default)]
struct Book {
    id: Uuid,
    master_id: Option<Uuid>,
    location_id: Option<Uuid>,
    author: String,
    title: String,
    lccn: String,
    isbn: String,
    publish_date: String,
}

struct BookFromQuery {
    master_book_id: Uuid,
    author: Option<String>,
    title: Option<String>,
    lccn: Option<String>,
    isbn: Option<String>,
    publish_date: Option<String>,
}

impl BookFromQuery {
    fn to_book(self) -> Book {
        Book { 
            id: self.master_book_id,
            master_id: None,
            location_id: None,
            author: self.author.unwrap_or("".to_string()),
            title: self.title.unwrap_or("".to_string()), 
            lccn: self.lccn.unwrap_or("".to_string()),
            isbn: self.isbn.unwrap_or("".to_string()),
            publish_date: self.publish_date.unwrap_or("".to_string()) 
        }
    }
}

#[derive(serde::Deserialize)]
struct BookQuery {
    lccn: String,
    isbn: String,
    title: String,
    author: String,
    publish_date: String,
}

#[derive(serde::Serialize, Default)]
struct BooksQuery {
    books: Vec<Book>,
}

#[derive(serde::Deserialize)]
struct BookUpdateQuery {
    id: Uuid,
    lccn: Option<String>,
    isbn: Option<String>,
    title: Option<String>,
    author: Option<String>,
    publish_date: Option<String>,
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
    State(api_context): State<ApiContext>
) -> (StatusCode, Json<BooksQuery>) {
    let connection_pool = &api_context.db;

    let book_list = sqlx::query!(
        r#"select * from "master_book" limit 200"#,
    ).fetch_all(connection_pool).await.expect("couldnt pull books");

    let mut books_q = vec![];
    for book in book_list.iter() {
        let book_b = Book {
            id: book.master_book_id,
            location_id: None,
            master_id: None,
            isbn: book.isbn.to_owned().unwrap_or("".to_string()),
            lccn: book.lccn.to_owned().unwrap_or("".to_string()),
            title: book.title.to_owned().unwrap_or("".to_string()),
            author: book.author.to_owned().unwrap_or("".to_string()),
            publish_date: book.publish_date.to_owned().unwrap_or("".to_string())
        };
        books_q.push(book_b);
    }
    //let booksQ = BooksQuery { books:  };
    (StatusCode::OK, Json(BooksQuery { books: books_q}))
}

async fn get_book(
    State(api_context): State<ApiContext>,
    request: Json<BookBody<GetBookQuery>>,
) -> Result<(StatusCode, Json<Book>), (StatusCode, String)> {
    let connection_pool = &api_context.db;
    let id = &request.book.id;
    let book = sqlx::query!(
        r#"select * from "master_book" where master_book_id = $1"#,
        id
    )
    .fetch_one(connection_pool)
    .await
    .expect("failed to get book");
    Ok((StatusCode::OK, Json(Book {
        id: book.master_book_id,
        master_id: None,
        location_id: None,
        title: book.title.unwrap_or("".to_string()),
        author: book.author.unwrap_or("".to_string()),
        lccn: book.lccn.unwrap_or("".to_string()),
        isbn: book.isbn.unwrap_or("".to_string()),
        publish_date: book.publish_date.unwrap_or("".to_string())

    })))
}

async fn create_book(
    State(api_context): State<ApiContext>,
    request: Json<BookBody<BookQuery>>
) -> (StatusCode, String) {
    let connection_pool = &api_context.db;
    let book_id = sqlx::query_scalar!(
        // language=PostgreSQL
        r#"insert into "master_book" (author, title, lccn, isbn, publish_date) values ($1, $2, $3, $4, $5) returning master_book_id"#,
        request.book.author,
        request.book.title,
        request.book.lccn,
        request.book.isbn,
        request.book.publish_date
    )
    .fetch_one(connection_pool)
    .await
    .expect("failed to insert into master_book");

    println!("{:?}", &request.book.lccn);
    println!("{:?}", &request.book.isbn);
    println!("{:?}", &request.book.title);
    println!("{:?}", &request.book.author);
    println!("{:?}", &request.book.publish_date);
    (StatusCode::OK, "Book Created".to_string())
}

async fn update_book(
    State(api_context): State<ApiContext>,
    Json(request): Json<BookBody<BookUpdateQuery>>,
) -> Result<(StatusCode, Json<Book>), (StatusCode, String)> {
    let connection_pool = &api_context.db;
    let author = &request.book.author.unwrap_or("".to_string());
    let updated_book = sqlx::query_as!(
        BookFromQuery,
        r#"
        with updated_book as (

        update "master_book" 
           set 
               author = coalesce($2, author),
               title = coalesce($3, title),
               lccn = coalesce($4, lccn),
               isbn = coalesce($5, isbn),
               publish_date = coalesce($6, publish_date)
        where master_book_id = $1
        returning 
                master_book_id, author, title, lccn, isbn, publish_date
        )
        select 
           updated_book.master_book_id master_book_id,
           updated_book.author author,
           updated_book.title title,
           updated_book.lccn lccn,
           updated_book.isbn isbn,
           updated_book.publish_date publish_date
        from updated_book    
        "#,
        request.book.id,
        author,
        request.book.title.unwrap_or("".to_string()),
        request.book.lccn.unwrap_or("".to_string()),
        request.book.isbn.unwrap_or("".to_string()),
        request.book.publish_date.unwrap_or("".to_string()),
    )
    .fetch_one(connection_pool)
    .await
    .expect("failed to update and fetch");

    let book = updated_book.to_book();

    Ok((StatusCode::OK, Json(book)))
}

async fn delete_book(
    State(api_context): State<ApiContext>,
    request: Json<BookBody<DeleteBook>>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let connection_pool = &api_context.db;

    let result = sqlx::query!(
        r#"delete from "master_book" where master_book_id=$1"#,
        request.book.id
    ).execute(connection_pool).await.expect("couldnt delete books");

    Ok((StatusCode::OK, "Book Deleted".to_string()))
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
            ServiceBuilder::new()
                .layer(
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
