use crate::schema::Shop;
use crate::{Error, Result};

use postgres::Row;
use rust_decimal::Decimal;
use std::convert::TryFrom;
use serde::Serialize;

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
    pub fn list(db: &mut postgres::Client) -> Result<Vec<Product>> {
        db.query(
            "SELECT slug, shop_slug, name, price, available, sold, details, picture
            FROM products",
            &[],
        )?
        .into_iter()
        .map(Product::try_from)
        .collect()
    }
    pub fn from_shop(db: &mut postgres::Client, shop: &Shop) -> Result<Vec<Product>> {
        db.query(
            "SELECT slug, shop_slug, name, price, available, sold, details, picture
            FROM products
            WHERE shop_slug = $1",
            &[&shop.slug],
        )?
        .into_iter()
        .map(Product::try_from)
        .collect()
    }
    pub fn delete(self, db: &mut postgres::Client) -> Result<()> {
        db.execute(
            "DELETE FROM products
            WHERE slug = $1",
            &[&self.slug],
        )?;
        Ok(())
    }
    pub fn modify(
        self,
        db: &mut postgres::Client,
        new_slug: Option<&str>,
        new_shop_slug: Option<&str>,
        new_name: Option<&str>,
        new_price: Option<&Decimal>,
        new_available: Option<i32>,
        new_sold: Option<i32>,
        new_details: Option<&str>,
        new_picture: Option<&str>,
    ) -> Result<Product> {
        let mut product = self;
        let old_slug = product.slug.clone();
        if let Some(x) = new_slug {
            product.slug = x.into();
        }
        if let Some(x) = new_shop_slug {
            product.shop_slug = x.into();
        }
        if let Some(x) = new_name {
            product.name = x.into();
        }
        if let Some(x) = new_price {
            product.price = x.clone();
        }
        if let Some(x) = new_available {
            product.available = x;
        }
        if let Some(x) = new_sold {
            product.sold = x;
        }
        if let Some(x) = new_details {
            product.details = x.into();
        }
        if let Some(x) = new_picture {
            product.picture = x.into();
        }
        db.execute(
            "UPDATE products SET slug = $1, shop_slug = $2, name = $3, price = $4, available = $5, sold = $6, details = $7, picture = $8
            WHERE slug = $9",
            &[
                &product.slug,
                &product.shop_slug,
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
            Error::builder_from(e)
                .description("Não foi possível atualizar informações")
        })?;
        Ok(product)
    }
}
