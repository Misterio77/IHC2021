use cincobola_backend::{routes, Database, Result};

use std::collections::HashMap;

use rocket::{routes, get};
use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;

#[get("/")]
fn home() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    Template::render("cincobola-home", &context)
}

#[rocket::main]
async fn main() -> Result<()> {
    rocket::build()
        .attach(Template::fairing())
        .attach(Database::fairing())
        .mount("/static", FileServer::from(relative!("static")))
        .mount("/", routes![home])
        .mount("/session", routes::session::routes())
        .mount("/users", routes::users::routes())
        .mount("/shops", routes::shops::routes())
        .mount("/products", routes::products::routes())
        .mount("/purchases", routes::purchases::routes())
        .launch()
        .await?;
    Ok(())
}
