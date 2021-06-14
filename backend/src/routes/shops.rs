use crate::schema::{Shop, User, UserToken};
use crate::{BodyResult, Database, Error, Result};
use futures::try_join;
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
    let token = token?;
    let requester = db.run(move |db| User::from_token(db, token));
    let target = db.run(move |db| User::from_email(db, &owner));

    let (requester, target) = try_join!(requester, target)?;

    if requester.email == target.email || requester.admin {
        let shops = db.run(move |db| Shop::from_user(db, &target));
        Ok(Json(shops.await?))
    } else {
        Err(Error::builder()
            .code(Status::Unauthorized)
            .description("Você não tem permissão para listar as lojas desse usuário")
            .build())
    }
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
    let token = token?;
    let requester = db.run(move |db| User::from_token(db, token)).await?;
    let shop = db
        .run(move |db| Shop::create(db, &body.slug, &body.name, &body.color, &requester.email))
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
    let token = token?;
    let requester = db.run(move |db| User::from_token(db, token));
    let shop = db.run(move |db| Shop::from_slug(db, &slug));
    let color = body.color.clone().map(|c| c.replace("#", ""));
    let (shop, requester) = try_join!(shop, requester)?;

    if requester.email == shop.owner_email || requester.admin {
        let shop = db.run(move |db| {
            shop.modify(
                db,
                body.slug.as_deref(),
                body.name.as_deref(),
                color.as_deref(),
                body.owner.as_deref(),
            )
        });
        Ok(Json(shop.await?))
    } else {
        Err(Error::builder()
            .code(Status::Unauthorized)
            .description("Você não tem permissão para modificar essa loja")
            .build())
    }
}

#[delete("/<slug>")]
async fn delete(db: Database, slug: String, token: Result<UserToken>) -> Result<status::NoContent> {
    let token = token?;
    let requester = db.run(move |db| User::from_token(db, token));
    let shop = db.run(move |db| Shop::from_slug(db, &slug));
    let (shop, requester) = try_join!(shop, requester)?;

    if requester.email == shop.owner_email || requester.admin {
        db.run(move |db| shop.delete(db)).await?;
        Ok(status::NoContent)
    } else {
        Err(Error::builder()
            .code(Status::Unauthorized)
            .description("Você não tem permissão para remover essa loja")
            .build())
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![list_by_owner, list, read, create, update, delete]
}
