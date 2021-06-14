use crate::schema::{Product, Shop, User, UserToken};
use crate::{BodyResult, Database, Error, Result};
use futures::try_join;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put};
use rust_decimal::Decimal;
use serde::Deserialize;

#[get("/?<shop>")]
async fn list_by_shop(db: Database, shop: String) -> Result<Json<Vec<Product>>> {
    let shop = Shop::read(&db, &shop).await?;
    let products = Product::list_from_shop(&db, &shop).await?;
    Ok(Json(products))
}

#[get("/")]
async fn list(db: Database) -> Result<Json<Vec<Product>>> {
    let products = Product::list(&db).await?;
    Ok(Json(products))
}

#[get("/<slug>")]
async fn read(db: Database, slug: String) -> Result<Json<Product>> {
    let product = Product::read(&db, &slug).await?;
    Ok(Json(product))
}

#[derive(Debug, Deserialize)]
struct CreateRequest {
    slug: String,
    shop_slug: String,
    name: String,
    price: Decimal,
    available: i32,
    sold: i32,
    details: String,
    picture: String,
}

#[post("/", data = "<body>")]
async fn create(
    db: Database,
    token: Result<UserToken>,
    body: BodyResult<'_, CreateRequest>,
) -> Result<status::Created<Json<Product>>> {
    let body = body?.into_inner();
    let requester = User::read_from_token(&db, &token?).await?;
    let requester_shops = Shop::list_from_user(&db, &requester).await?;
    let product = Product {
        slug: body.slug,
        shop_slug: body.shop_slug,
        name: body.name,
        price: body.price,
        available: body.available,
        sold: body.sold,
        details: body.details,
        picture: body.picture,
    };
    // Caso esse usuário não tenha permissão de adicionar produtos à essa loja
    if !requester_shops
        .iter()
        .any(|shop| shop.slug == product.shop_slug)
        && !requester.admin
    {
        return Err(Error::builder()
            .code(Status::Unauthorized)
            .description("Você não tem permissão ppara adicionar produtos à essa loja")
            .build());
    }

    product.create(&db).await?;
    Ok(status::Created::new(format!(
        "https://cincobola.misterio.me/products/{}",
        product.slug
    ))
    .body(Json(product)))
}

#[derive(Debug, Deserialize)]
struct UpdateRequest {
    slug: Option<String>,
    shop_slug: Option<String>,
    name: Option<String>,
    price: Option<Decimal>,
    available: Option<i32>,
    sold: Option<i32>,
    details: Option<String>,
    picture: Option<String>,
}

#[put("/<slug>", data = "<body>")]
async fn update(
    db: Database,
    slug: String,
    token: Result<UserToken>,
    body: BodyResult<'_, UpdateRequest>,
) -> Result<Json<Product>> {
    let body = body?.into_inner();
    let token = token?;
    let requester = User::read_from_token(&db, &token);
    let product = Product::read(&db, &slug);
    let (requester, mut product) = try_join!(requester, product)?;
    let requester_shops = Shop::list_from_user(&db, &requester).await?;

    if !requester_shops
        .iter()
        .any(|shop| shop.slug == product.shop_slug)
        && !requester.admin
    {
        return Err(Error::builder()
            .code(Status::Forbidden)
            .description("Você não tem permissão para modificar esse produto")
            .build());
    }

    let old_slug = product.slug.clone();
    // Adicionar campos
    if let Some(x) = body.slug {
        product.slug = x;
    }
    if let Some(x) = body.shop_slug {
        product.shop_slug = x;
    }
    if let Some(x) = body.name {
        product.name = x;
    }
    if let Some(x) = body.price {
        product.price = x;
    }
    if let Some(x) = body.available {
        product.available = x;
    }
    if let Some(x) = body.sold {
        product.sold = x;
    }
    if let Some(x) = body.details {
        product.details = x;
    }
    if let Some(x) = body.picture {
        product.picture = x;
    }

    if !requester_shops
        .iter()
        .any(|shop| shop.slug == product.shop_slug)
        && !requester.admin
    {
        return Err(Error::builder()
            .code(Status::Forbidden)
            .description("Você não tem permissão para mover o produto à uma loja que não é sua")
            .build());
    }

    product.update(&db, &old_slug).await?;
    Ok(Json(product))
}

#[delete("/<slug>")]
async fn delete(db: Database, slug: String, token: Result<UserToken>) -> Result<status::NoContent> {
    let token = token?;
    let requester = User::read_from_token(&db, &token);
    let product = Product::read(&db, &slug);
    let (requester, product) = try_join!(requester, product)?;
    let requester_shops = Shop::list_from_user(&db, &requester).await?;

    if !requester_shops
        .iter()
        .any(|shop| shop.slug == product.shop_slug)
        && !requester.admin
    {
        return Err(Error::builder()
            .code(Status::Forbidden)
            .description("Você não tem permissão para remover esse produto")
            .build());
    }
    product.delete(&db).await?;
    Ok(status::NoContent)
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![delete, update, create, read, list, list_by_shop]
}
