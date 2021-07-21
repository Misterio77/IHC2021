use crate::schema::{Product, Purchase, Shop, User, UserToken};
use crate::{BodyResult, Database, Error, Result};
use futures::try_join;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use serde::Deserialize;
use chrono::Utc;

#[get("/?<purchaser>")]
async fn list_by_purchaser(
    db: Database,
    purchaser: String,
    token: Result<UserToken>,
) -> Result<Json<Vec<Purchase>>> {
    let token = token?;
    let requester = User::read_from_token(&db, &token);
    let target = User::read(&db, &purchaser);

    let (requester, target) = try_join!(requester, target)?;

    if requester.email != target.email && !requester.admin {
        return Err(Error::builder()
            .code(Status::Forbidden)
            .description("Você não tem permissão para listar as compras desse usuário")
            .build());
    }

    let purchases = Purchase::list_from_user(&db, &target);
    Ok(Json(purchases.await?))
}

#[get("/?<product>", rank = 2)]
async fn list_by_product(
    db: Database,
    product: String,
    token: Result<UserToken>,
) -> Result<Json<Vec<Purchase>>> {
    let token = token?;
    let requester = User::read_from_token(&db, &token);
    let target = Product::read(&db, &product);

    let (requester, target) = try_join!(requester, target)?;
    let requester_shops = Shop::list_from_user(&db, &requester).await?;

    if !requester_shops
        .iter()
        .any(|shop| shop.slug == target.shop)
        && !requester.admin
    {
        return Err(Error::builder()
            .code(Status::Unauthorized)
            .description("Você não tem permissão para listar compras desse produto")
            .build());
    }
    let purchases = Purchase::list_from_product(&db, &target).await?;
    Ok(Json(purchases))
}

#[get("/?<shop>", rank = 3)]
async fn list_by_shop(
    db: Database,
    shop: String,
    token: Result<UserToken>,
) -> Result<Json<Vec<Purchase>>> {
    let token = token?;
    let requester = User::read_from_token(&db, &token);
    let target = Shop::read(&db, &shop);

    let (requester, target) = try_join!(requester, target)?;
    if requester.email != target.manager && !requester.admin {
        return Err(Error::builder()
            .code(Status::Unauthorized)
            .description("Você não tem permissão para listar compras dessa loja")
            .build());
    }

    let purchases = Purchase::list_from_shop(&db, &target).await?;
    Ok(Json(purchases))
}

#[get("/", rank = 100)]
async fn list(db: Database, token: Result<UserToken>) -> Result<Json<Vec<Purchase>>> {
    let token = token?;
    let requester = User::read_from_token(&db, &token).await?;
    if !requester.admin {
        return Err(Error::builder()
            .code(Status::Forbidden)
            .description("Você não tem permissão para listar todas as compras")
            .build());
    }
    let purchases = Purchase::list(&db).await?;
    Ok(Json(purchases))
}

#[derive(Deserialize, Debug)]
struct BuyRequest {
    amount: i32,
    product: String,
}

#[post("/", data = "<body>")]
async fn create(db: Database, token: Result<UserToken>, body: BodyResult<'_, BuyRequest>) -> Result<Json<Purchase>> {
    let token = token?;
    let body = body?.into_inner();

    let (requester, product) = try_join!(User::read_from_token(&db, &token),Product::read(&db, &body.product))?;

    let purchase = Purchase {
        amount: body.amount,
        product: Some(product.slug),
        purchaser: Some(requester.email),
        paid: product.price,
        time: Utc::now()
    };
    
    // Aqui a gente cobraria a pessoa

    purchase.create(&db).await?;

    Ok(Json(purchase))
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![list, list_by_shop, list_by_purchaser, list_by_product, create]
}
