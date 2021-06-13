pub mod error;
pub use error::{Error, Result};

pub mod routes;
pub mod schema;

use rocket_sync_db_pools::database;
#[database("database")]
/// Database do backend
pub struct Database(postgres::Client);

/// Result para facilitar obteção de Json no body
pub type BodyResult<'a, T> =
    std::result::Result<rocket::serde::json::Json<T>, rocket::serde::json::Error<'a>>;
