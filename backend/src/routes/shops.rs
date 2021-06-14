use crate::schema::{Shop, User, UserToken};
use crate::{BodyResult, Database, Error, Result};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put};
use serde::Deserialize;

#[get("/?<owner>")]
async fn list_by_owner(
    db: Database,
    owner: String,
    token: Result<UserToken>,
) -> Result<Json<Vec<Shop>>> {
    let shops = db
        .run(move |db| -> Result<Vec<Shop>> {
            let user = User::from_token(db, token?)?;
            if owner != user.email {
                return Err(Error::builder()
                    .code(Status::Unauthorized)
                    .description("Você não tem permissão para listar as lojas desse usuário")
                    .build());
            }
            Shop::from_user(db, &user)
        })
        .await?;
    Ok(Json(shops))
}

#[get("/")]
async fn list(db: Database) -> Result<Json<Vec<Shop>>> {
    let shops = db
        .run(move |db| -> Result<Vec<Shop>> { Shop::list(db) })
        .await?;
    Ok(Json(shops))
}

#[get("/<slug>")]
async fn read(db: Database, slug: String) -> Result<Json<Shop>> {
    let shop = db
        .run(move |db| -> Result<Shop> { Shop::from_slug(db, &slug) })
        .await?;
    Ok(Json(shop))
}

#[derive(Debug, Deserialize)]
struct CreateRequest {
    slug: String,
    name: String,
    color: String,
}

#[post("/", data = "<body>")]
async fn create(
    db: Database,
    token: Result<UserToken>,
    body: BodyResult<'_, CreateRequest>,
) -> Result<status::Created<Json<Shop>>> {
    let body = body?;
    let shop = db
        .run(move |db| -> Result<Shop> {
            let user = User::from_token(db, token?)?;
            let color = body.color.replace("#", "");
            Shop::create(&user, db, &body.slug, &body.name, &color)
        })
        .await?;
    Ok(
        status::Created::new(format!("https://cincobola.misterio.me/shops/{}", shop.slug))
            .body(Json(shop)),
    )
}

#[derive(Debug, Deserialize)]
struct UpdateRequest {
    slug: Option<String>,
    name: Option<String>,
    color: Option<String>,
    owner: Option<String>,
}

#[put("/<slug>", data = "<body>")]
async fn update(
    db: Database,
    slug: String,
    token: Result<UserToken>,
    body: BodyResult<'_, UpdateRequest>,
) -> Result<Json<Shop>> {
    let body = body?;
    let shop = db
        .run(move |db| -> Result<Shop> {
            let user = User::from_token(db, token?)?;
            let shop = Shop::from_slug(db, &slug)?;
            let color = body.color.clone().map(|c| c.replace("#", ""));
            shop.modify(
                db,
                &user,
                body.slug.as_deref(),
                body.name.as_deref(),
                color.as_deref(),
                body.owner.as_deref(),
            )
        })
        .await?;
    Ok(Json(shop))
}

#[delete("/<slug>")]
async fn delete(
    db: Database,
    slug: String,
    token: Result<UserToken>,
) -> Result<status::NoContent> {
    db.run(move |db| -> Result<()> {
        let user = User::from_token(db, token?)?;
        let shop = Shop::from_slug(db, &slug)?;
        shop.delete(db, &user)
    }).await?;
    Ok(status::NoContent)
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![list_by_owner, list, read, create, update, delete]
}
