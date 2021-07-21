use crate::schema::Shop;
use crate::{Database, Error, Result};

use postgres::Row;
use rocket::http::Status;
use rust_decimal::Decimal;
use serde::Serialize;
use std::convert::{TryFrom, TryInto};

#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct Product {
    pub slug: String,
    pub shop: String,
    pub name: String,
    pub price: Decimal,
    pub available: i32,
    pub sold: i32,
    pub details: String,
    pub picture: String,
}

impl TryFrom<Row> for Product {
    type Error = Error;
    fn try_from(row: Row) -> Result<Self> {
        Ok(Self {
            slug: row.try_get("slug")?,
            shop: row.try_get("shop")?,
            name: row.try_get("name")?,
            price: row.try_get("price")?,
            available: row.try_get("available")?,
            sold: row.try_get("sold")?,
            details: row.try_get("details")?,
            picture: row.try_get("picture")?,
        })
    }
}

impl Product {
    pub async fn read(db: &Database, slug: &str) -> Result<Product> {
        let slug: String = slug.into();
        db.run(move |db| {
            db.query_one(
                "SELECT *
                FROM products
                WHERE slug = $1",
                &[&slug],
            )
            .map_err(|e| {
                Error::builder_from(e)
                    .code(Status::NotFound)
                    .description("Produto não encontrado")
            })
        })
        .await?
        .try_into()
    }
    pub async fn list(db: &Database) -> Result<Vec<Product>> {
        db.run(move |db| {
            db.query(
                "SELECT *
                FROM products",
                &[],
            )
        })
        .await?
        .into_iter()
        .map(Product::try_from)
        .collect()
    }
    pub async fn list_from_shop(db: &Database, shop: &Shop) -> Result<Vec<Product>> {
        let shop = shop.clone();
        db.run(move |db| {
            db.query(
                "SELECT *
                FROM products
                WHERE shop = $1",
                &[&shop.slug],
            )
        })
        .await?
        .into_iter()
        .map(Product::try_from)
        .collect()
    }
    pub async fn delete(&self, db: &Database) -> Result<()> {
        let product = self.clone();
        db.run(move |db| {
            db.execute(
                "DELETE FROM products
                WHERE slug = $1",
                &[&product.slug],
            )
        })
        .await?;
        Ok(())
    }
    pub async fn update(&self, db: &Database, old_slug: &str) -> Result<()> {
        let product = self.clone();
        let old_slug: String = old_slug.into();
        db.run(move |db| {
            db.execute(
                "UPDATE products
                SET slug = $1,
                shop = $2,
                name = $3,
                price = $4,
                available = $5,
                sold = $6,
                details = $7,
                picture = $8
                WHERE slug = $9",
                &[
                    &product.slug,
                    &product.shop,
                    &product.name,
                    &product.price,
                    &product.available,
                    &product.sold,
                    &product.details,
                    &product.picture,
                    &old_slug,
                ],
            )
            .map_err(|e| {
                Error::builder_from(e).description("Não foi possível atualizar informações")
            })
        })
        .await?;
        Ok(())
    }
    pub async fn create(&self, db: &Database) -> Result<()> {
        let product = self.clone();
        db.run(move |db| {
            db.execute(
                "INSERT INTO products
                (slug, shop, name, price, available, sold, details, picture)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    &product.slug,
                    &product.shop,
                    &product.name,
                    &product.price,
                    &product.available,
                    &product.sold,
                    &product.details,
                    &product.picture,
                ],
            )
            .map_err(|e| {
                Error::builder_from(e)
                    .code(Status::BadRequest)
                    .description("O identificador especificado já está registrado")
            })
        })
        .await?;
        Ok(())
    }
}
