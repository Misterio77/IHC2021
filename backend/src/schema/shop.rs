use crate::{Error, Result};
use postgres::Row;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct Shop {
    pub slug: String,
    pub name: String,
    pub color: String,
    pub owner_email: String,
}

impl TryFrom<Row> for Shop {
    type Error = Error;
    fn try_from(row: Row) -> Result<Self> {
        Ok(Self {
            slug: row.try_get("slug")?,
            name: row.try_get("name")?,
            color: row.try_get("color")?,
            owner_email: row.try_get("owner_email")?,
        })
    }
}
