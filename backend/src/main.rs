use cincobola_backend::{routes, Database, Result};

#[rocket::main]
async fn main() -> Result<()> {
    rocket::build()
        .attach(Database::fairing())
        .mount("/sessions", routes::sessions::routes())
        .mount("/users", routes::users::routes())
        .mount("/shops", routes::shops::routes())
        .launch()
        .await?;
    Ok(())
}
