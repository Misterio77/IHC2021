use crate::schema::User;
use crate::{Error, Result};

use postgres::Row;
use rocket::http::Status;
use serde::Serialize;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Serialize)]
pub struct Shop {
    pub slug: String,
    pub name: String,
    pub color: String,
    #[serde(skip_serializing)]
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

impl Shop {
    pub fn list(db: &mut postgres::Client) -> Result<Vec<Shop>> {
        db.query(
            "SELECT slug, name, color, owner_email
            FROM shops",
            &[],
        )?
        .into_iter()
        .map(Shop::try_from)
        .collect()
    }
    pub fn from_slug(db: &mut postgres::Client, slug: &str) -> Result<Shop> {
        db.query_one(
            "SELECT slug, name, color, owner_email
            FROM shops
            WHERE slug = $1",
            &[&slug],
        )
        .map_err(|e| {
            Error::builder_from(e)
                .code(Status::NotFound)
                .description("Loja não encontrada")
        })?
        .try_into()
    }
    pub fn from_user(db: &mut postgres::Client, user: &User) -> Result<Vec<Shop>> {
        db.query(
            "SELECT slug, name, color, owner_email
            FROM shops
            WHERE owner_email = $1",
            &[&user.email],
        )?
        .into_iter()
        .map(Shop::try_from)
        .collect()
    }
    pub fn delete(self, db: &mut postgres::Client) -> Result<()> {
        db.execute(
            "DELETE FROM shops
            WHERE slug = $1",
            &[&self.slug],
        )?;
        Ok(())
    }
    pub fn modify(
        self,
        db: &mut postgres::Client,
        new_slug: Option<&str>,
        new_name: Option<&str>,
        new_color: Option<&str>,
        new_owner_email: Option<&str>,
    ) -> Result<Shop> {
        let mut shop = self;
        let old_slug = shop.slug.clone();
        if let Some(new_slug) = new_slug {
            shop.slug = new_slug.into();
        }
        if let Some(new_name) = new_name {
            shop.name = new_name.into();
        }
        if let Some(new_color) = new_color {
            shop.color = new_color.into();
        }
        if let Some(new_owner_email) = new_owner_email {
            shop.owner_email = new_owner_email.into();
        }

        db.execute(
            "UPDATE shops SET slug = $1, name = $2, color = $3, owner_email = $4
            WHERE slug = $5",
            &[
                &shop.slug,
                &shop.name,
                &shop.color,
                &shop.owner_email,
                &old_slug,
            ],
        )
        .map_err(|e| {
            Error::builder_from(e)
                .description("Não foi possível atualizar informações")
        })?;
        Ok(shop)
    }
    pub fn create(
        db: &mut postgres::Client,
        slug: &str,
        name: &str,
        color: &str,
        owner: &str,
    ) -> Result<Shop> {
        let shop = Shop {
            slug: slug.into(),
            name: name.into(),
            color: color.into(),
            owner_email: owner.into(),
        };
        db.execute(
            "INSERT INTO shops (slug, name, color, owner_email) VALUES ($1, $2, $3, $4)",
            &[&shop.slug, &shop.name, &shop.color, &shop.owner_email],
        )
        .map_err(|e| {
            Error::builder_from(e)
                .code(Status::BadRequest)
                .description("Uma loja com esse identificador já existe")
        })?;
        Ok(shop)
    }
}
