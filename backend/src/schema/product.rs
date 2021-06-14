use crate::schema::Shop;
use crate::{Error, Result};

use postgres::Row;
use rocket::http::Status;
use rust_decimal::Decimal;
use serde::Serialize;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Serialize)]
pub struct Product {
    pub slug: String,
    pub shop_slug: String,
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
            shop_slug: row.try_get("shop_slug")?,
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
    pub fn read(db: &mut postgres::Client, slug: &str) -> Result<Product> {
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
        })?
        .try_into()
    }
    pub fn list(db: &mut postgres::Client) -> Result<Vec<Product>> {
        db.query(
            "SELECT *
            FROM products",
            &[],
        )?
        .into_iter()
        .map(Product::try_from)
        .collect()
    }
    pub fn list_from_shop(db: &mut postgres::Client, shop: &Shop) -> Result<Vec<Product>> {
        db.query(
            "SELECT *
            FROM products
            WHERE shop_slug = $1",
            &[&shop.slug],
        )?
        .into_iter()
        .map(Product::try_from)
        .collect()
    }
    pub fn delete(&self, db: &mut postgres::Client) -> Result<()> {
        db.execute(
            "DELETE FROM products
            WHERE slug = $1",
            &[&self.slug],
        )?;
        Ok(())
    }
    pub fn update(&self, old_slug: &str, db: &mut postgres::Client) -> Result<()> {
        db.execute(
            "UPDATE products SET slug = $1, shop_slug = $2, name = $3, price = $4, available = $5, sold = $6, details = $7, picture = $8
            WHERE slug = $9",
            &[
                &self.slug,
                &self.shop_slug,
                &self.name,
                &self.price,
                &self.available,
                &self.sold,
                &self.details,
                &self.picture,
                &old_slug,
            ],
        )
        .map_err(|e| {
            Error::builder_from(e)
                .description("Não foi possível atualizar informações")
        })?;
        Ok(())
    }
}
