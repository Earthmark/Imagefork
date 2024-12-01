pub mod creator;
pub mod creator_token;
pub mod poster;

use rocket_db_pools::{diesel, Database};

pub use creator::Creator;
pub use creator_token::CreatorToken;
pub use poster::Poster;

#[derive(Database)]
#[database("imagefork")]
pub struct Imagefork(pub diesel::PgPool);
