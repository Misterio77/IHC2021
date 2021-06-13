use crate::{Error, Result};
use postgres::Row;
use rust_decimal::Decimal;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct Product {
    pub slug: String,
    pub shop_slug: String,
    pub name: String,
    pub price: Decimal,
    pub available: Option<i32>,
    pub sold: i32,
    pub details: Option<String>,
    pub picture: Option<String>,
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
