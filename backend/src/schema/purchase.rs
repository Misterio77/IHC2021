use crate::schema::{Product, Shop, User};
use crate::{Database, Error, Result};

use chrono::{DateTime, Utc};
use postgres::Row;
use rocket::http::Status;
use rust_decimal::Decimal;
use serde::Serialize;
use std::convert::{TryFrom, TryInto};

#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
pub struct Purchase {
    pub amount: i32,
    pub paid: Decimal,
    pub time: DateTime<Utc>,
    pub product: Option<String>,
    pub purchaser: Option<String>,
}

impl TryFrom<Row> for Purchase {
    type Error = Error;
    fn try_from(row: Row) -> Result<Self> {
        Ok(Self {
            amount: row.try_get("amount")?,
            paid: row.try_get("paid")?,
            time: row.try_get("time")?,
            product: row.try_get("product")?,
            purchaser: row.try_get("purchaser")?,
        })
    }
}

impl Purchase {
    pub async fn read(db: &Database, time: DateTime<Utc>) -> Result<Purchase> {
        db.run(move |db| {
            db.query_one(
                "SELECT *
                FROM purchases
                WHERE time = $1",
                &[&time],
            )
            .map_err(|e| {
                Error::builder_from(e)
                    .code(Status::NotFound)
                    .description("Compra não encontrada")
            })
        })
        .await?
        .try_into()
    }
    pub async fn list(db: &Database) -> Result<Vec<Purchase>> {
        db.run(move |db| {
            db.query(
                "SELECT *
                FROM purchases",
                &[],
            )
        })
        .await?
        .into_iter()
        .map(Purchase::try_from)
        .collect()
    }
    pub async fn list_from_shop(db: &Database, shop: &Shop) -> Result<Vec<Purchase>> {
        let shop = shop.clone();
        db.run(move |db| {
            db.query(
                "SELECT *
                FROM purchases
                INNER JOIN products
                ON purchases.product = products.slug
                WHERE products.shop = $1",
                &[&shop.slug],
            )
        })
        .await?
        .into_iter()
        .map(Purchase::try_from)
        .collect()
    }
    pub async fn list_from_product(db: &Database, product: &Product) -> Result<Vec<Purchase>> {
        let product = product.clone();
        db.run(move |db| {
            db.query(
                "SELECT *
                FROM purchases
                WHERE product = $1",
                &[&product.slug],
            )
        })
        .await?
        .into_iter()
        .map(Purchase::try_from)
        .collect()
    }
    pub async fn list_from_user(db: &Database, user: &User) -> Result<Vec<Purchase>> {
        let user = user.clone();
        db.run(move |db| {
            db.query(
                "SELECT *
                FROM purchases
                WHERE purchaser = $1",
                &[&user.email],
            )
        })
        .await?
        .into_iter()
        .map(Purchase::try_from)
        .collect()
    }
    pub async fn delete(&self, db: &Database) -> Result<()> {
        let purchase = self.clone();
        db.run(move |db| {
            db.execute(
                "DELETE FROM purchases
                WHERE time = $1",
                &[&purchase.time],
            )
        })
        .await?;
        Ok(())
    }
    pub async fn update(&self, db: &Database, old_time: DateTime<Utc>) -> Result<()> {
        let purchase = self.clone();
        db.run(move |db| {
            db.execute(
                "UPDATE purchases SET
                amount = $1,
                paid = $2,
                time = $3,
                product = $4,
                purchaser = $5,
                WHERE time = $6",
                &[
                    &purchase.amount,
                    &purchase.paid,
                    &purchase.time,
                    &purchase.product,
                    &purchase.purchaser,
                    &old_time,
                ],
            )
            .map_err(|e| Error::builder_from(e).code(Status::BadRequest))
        })
        .await?;
        Ok(())
    }
    pub async fn create(&self, db: &Database) -> Result<()> {
        let purchase = self.clone();
        db.run(move |db| {
            db.execute(
                "INSERT INTO purchases
                (amount, paid, time, product, purchaser)
                VALUES ($1, $2, $3, $4, $5)",
                &[
                    &purchase.amount,
                    &purchase.paid,
                    &purchase.time,
                    &purchase.product,
                    &purchase.purchaser,
                ],
            )
            .map_err(|e| Error::builder_from(e).code(Status::BadRequest))
        })
        .await?;
        Ok(())
    }
}
