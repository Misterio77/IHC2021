use crate::{Error, Result};
use chrono::{DateTime, Utc};
use postgres::Row;
use rust_decimal::Decimal;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct Purchase {
    pub id: i32,
    pub amount: i32,
    pub paid: Decimal,
    pub time: DateTime<Utc>,
    pub product_slug: Option<String>,
    pub purchaser_email: Option<String>,
}

impl TryFrom<Row> for Purchase {
    type Error = Error;
    fn try_from(row: Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            amount: row.try_get("amount")?,
            paid: row.try_get("paid")?,
            time: row.try_get("time")?,
            product_slug: row.try_get("product_slug")?,
            purchaser_email: row.try_get("purchaser_email")?,
        })
    }
}
