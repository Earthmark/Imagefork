pub mod creator;
pub mod poster;
pub mod poster_image;

pub use poster::Poster;

pub type DbPool = sqlx::PgPool;

pub async fn build_pool(url: &str) -> crate::Result<DbPool> {
    Ok(sqlx::postgres::PgPoolOptions::new().connect(url).await?)
}
