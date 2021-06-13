use cincobola_backend::{routes, Database, Result};

#[rocket::main]
async fn main() -> Result<()> {
    rocket::build()
        .attach(Database::fairing())
        .mount("/sessions", routes::sessions::routes())
        .mount("/users", routes::users::routes())
        .launch()
        .await?;
    Ok(())
}
