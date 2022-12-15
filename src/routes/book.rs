use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use sqlx::types::Uuid;

use crate::db::Db;
use crate::routes::{ApiError, ApiContext};

/// Used to namespace our JSON query
/// { "book": <T> }
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BookBody<T = BookQuery> {
    pub book: T,
}

/// The base return structure for our book object to the client
#[derive(serde::Serialize, Default)]
pub struct Book {
    pub id: Uuid,
    pub master_id: Option<Uuid>,
    pub location_id: Option<Uuid>,
    pub author: Option<String>,
    pub title: Option<String>,
    pub lccn: Option<String>,
    pub isbn: Option<String>,
    pub publish_date: Option<String>,
}

/// Query coming from Client
#[derive(serde::Deserialize, Clone)]
pub struct BookQuery {
    pub lccn: String,
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub publish_date: String,
}

/// Array Query coming from Client
#[derive(serde::Serialize, Default)]
pub struct BooksQuery {
    pub books: Vec<Book>,
}

/// Update Query coming from Client
#[derive(serde::Deserialize, Clone)]
pub struct BookUpdateQuery {
    pub id: Uuid,
    pub lccn: Option<String>,
    pub isbn: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub publish_date: Option<String>,
}

/// Get Book Query coming from Client
#[derive(serde::Deserialize)]
pub struct GetBookQuery {
    pub id: Uuid,
}

/// Delete Book Query coming from Client
#[derive(serde::Deserialize)]
pub struct DeleteBook {
   pub id: Uuid,
}

/// Database Object to be cast into a Book
#[derive(Clone)]
pub struct BookFromQuery {
    pub master_book_id: Uuid,
    pub author: Option<String>,
    pub title: Option<String>,
    pub lccn: Option<String>,
    pub isbn: Option<String>,
    pub publish_date: Option<String>,
}

/// Casts the BookFromQuery to Book. Maybe implement Into Trait here
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

/// Gets a list of books 
pub async fn get_list_books(
    State(api_context): State<ApiContext>,
) -> Result<(StatusCode, Json<BooksQuery>), ApiError> {
    let connection_pool = &api_context.db;

    let list = Db::get_book_list(connection_pool).await?;
    let books = list.iter().map(|book| book.to_book()).collect();

    Ok((StatusCode::OK, Json(BooksQuery { books })))
}

/// Get a specific book 
pub async fn get_book(
    State(api_context): State<ApiContext>,
    request: Json<BookBody<GetBookQuery>>,
) -> Result<(StatusCode, Json<Book>), ApiError> {
    let connection_pool = &api_context.db;
    let id = &request.book.id;

    let book_from_query = Db::get_book(id, connection_pool).await?;

    Ok((StatusCode::OK, Json(book_from_query.to_book())))
}

/// Creates a book
pub async fn create_book(
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

/// Updates a book
pub async fn update_book(
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

/// Deletes a particular book
pub async fn delete_book(
    State(api_context): State<ApiContext>,
    request: Json<BookBody<DeleteBook>>,
) -> Result<(StatusCode, String), ApiError> {
    let connection_pool = &api_context.db;

    let result = Db::delete_book(request.book.id, connection_pool).await?;

    Ok((StatusCode::OK, result.rows_affected().to_string()))
}
