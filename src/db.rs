use sqlx::postgres::PgQueryResult;
use sqlx::types::Uuid;
use sqlx::PgPool;

use crate::routes::{BookFromQuery, BookQuery, BookUpdateQuery};

pub struct Db;

impl Db {
    pub async fn get_book_list(
        connection_pool: &PgPool,
    ) -> Result<Vec<BookFromQuery>, sqlx::Error> {
        sqlx::query_as!(
            BookFromQuery,
            r#"select 
                master_book_id, 
                author,
                title,
                lccn,
                isbn,
                publish_date
            from "master_book" limit 200"#
        )
        .fetch_all(connection_pool)
        .await
    }

    pub async fn get_book(
        id: &Uuid,
        connection_pool: &PgPool,
    ) -> Result<BookFromQuery, sqlx::Error> {
        sqlx::query_as!(
            BookFromQuery,
            r#"select 
                master_book_id, 
                author,
                title,
                lccn,
                isbn,
                publish_date
            from "master_book" where master_book_id = $1"#,
            id
        )
        .fetch_one(connection_pool)
        .await
    }

    pub async fn create_book(
        book_query: BookQuery,
        connection_pool: &PgPool,
    ) -> Result<Uuid, sqlx::Error> {
        sqlx::query_scalar!(
            // language=PostgreSQL
            r#"insert into "master_book" (author, title, lccn, isbn, publish_date) values ($1, $2, $3, $4, $5) returning master_book_id"#,
            book_query.author,
            book_query.title,
            book_query.lccn,
            book_query.isbn,
            book_query.publish_date
        )
        .fetch_one(connection_pool)
        .await
    }

    pub async fn update_book(
        book_update_query: BookUpdateQuery,
        connection_pool: &PgPool,
    ) -> Result<BookFromQuery, sqlx::Error> {
        sqlx::query_as!(
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
            book_update_query.id,
            book_update_query.author,
            book_update_query.title,
            book_update_query.lccn,
            book_update_query.isbn,
            book_update_query.publish_date,
        )
        .fetch_one(connection_pool)
        .await
    }

    pub async fn delete_book(
        id: Uuid,
        connection_pool: &PgPool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(r#"delete from "master_book" where master_book_id=$1"#, id)
            .execute(connection_pool)
            .await
    }
}
