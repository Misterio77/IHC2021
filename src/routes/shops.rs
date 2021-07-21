use crate::schema::{Shop, User, UserToken};
use crate::{BodyResult, Database, Error, Result};
use futures::try_join;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{delete, get, patch, post};
use serde::Deserialize;

#[get("/?<owner>")]
async fn list_by_owner(
    db: Database,
    owner: String,
    token: Result<UserToken>,
) -> Result<Json<Vec<Shop>>> {
    let token = token?;
    let requester = User::read_from_token(&db, &token);
    let target = User::read(&db, &owner);

    let (requester, target) = try_join!(requester, target)?;

    if requester.email != target.email && !requester.admin {
        return Err(Error::builder()
            .code(Status::Forbidden)
            .description("Você não tem permissão para listar as lojas desse usuário")
            .build());
    }

    let shops = Shop::list_from_user(&db, &target);
    Ok(Json(shops.await?))
}

#[get("/")]
async fn list(db: Database) -> Result<Json<Vec<Shop>>> {
    let shops = Shop::list(&db).await?;
    Ok(Json(shops))
}

#[get("/<slug>")]
async fn read(db: Database, slug: String) -> Result<Json<Shop>> {
    let shop = Shop::read(&db, &slug).await?;
    Ok(Json(shop))
}

#[derive(Debug, Deserialize)]
struct CreateRequest {
    slug: String,
    name: String,
    color: String,
    logo: String,
    owner: String,
}

#[post("/", data = "<body>")]
async fn create(
    db: Database,
    token: Result<UserToken>,
    body: BodyResult<'_, CreateRequest>,
) -> Result<status::Created<Json<Shop>>> {
    let body = body?.into_inner();
    let token = token?;
    let requester = User::read_from_token(&db, &token).await?;
    let shop = Shop {
        slug: body.slug,
        name: body.name,
        color: body.color.replace("#", ""),
        logo: body.logo,
        owner: body.owner,
    };

    if shop.color.len() > 6 {
        return Err(Error::builder()
            .code(Status::BadRequest)
            .description("A cor da loja deve ser heximadecimal e ter, no máximo, 6 caracteres")
            .build());
    }

    // Retornar erro caso o usuário esteja criando uma loja em um nome que não o dele
    if requester.email != shop.owner && !requester.admin {
        return Err(Error::builder()
            .code(Status::Forbidden)
            .description("Você não tem permissão para criar uma loja em nome de outra pessoa")
            .build());
    }

    shop.create(&db).await?;
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
    logo: Option<String>,
    owner: Option<String>,
}

#[patch("/<slug>", data = "<body>")]
async fn update(
    db: Database,
    slug: String,
    token: Result<UserToken>,
    body: BodyResult<'_, UpdateRequest>,
) -> Result<Json<Shop>> {
    let body = body?.into_inner();
    let token = token?;

    let requester = User::read_from_token(&db, &token);
    let shop = Shop::read(&db, &slug);

    let (mut shop, requester) = try_join!(shop, requester)?;
    let old_slug = shop.slug.clone();

    // Retornar erro caso o usuário esteja alterando uma loja que não é dele
    if requester.email != shop.owner && !requester.admin {
        return Err(Error::builder()
            .code(Status::Forbidden)
            .description("Você não tem permissão para modificar essa loja")
            .build());
    }

    // Adicionar campos
    if let Some(x) = body.slug {
        shop.slug = x;
    }
    if let Some(x) = body.name {
        shop.name = x;
    }
    if let Some(x) = body.color {
        shop.color = x.replace("#", "");
    }
    if let Some(x) = body.logo {
        shop.logo = x;
    }
    if let Some(x) = body.owner {
        shop.owner = x;
    }

    // Retornar erro caso o usuário esteja trocando posse da loja
    if requester.email != shop.owner && !requester.admin {
        return Err(Error::builder()
            .code(Status::Forbidden)
            .description("Você não tem permissão para trocar o dono de uma loja")
            .build());
    }

    shop.update(&db, &old_slug).await?;
    Ok(Json(shop))
}

#[delete("/<slug>")]
async fn delete(db: Database, slug: String, token: Result<UserToken>) -> Result<status::NoContent> {
    let token = token?;
    let requester = User::read_from_token(&db, &token);
    let shop = Shop::read(&db, &slug);
    let (shop, requester) = try_join!(shop, requester)?;

    if requester.email != shop.owner && !requester.admin {
        return Err(Error::builder()
            .code(Status::Forbidden)
            .description("Você não tem permissão para remover essa loja")
            .build());
    }
    shop.delete(&db).await?;
    Ok(status::NoContent)
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![list_by_owner, list, read, create, update, delete]
}
