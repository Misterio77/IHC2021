use crate::schema::User;
use crate::{Database, Error, Result};

use postgres::Row;
use rocket::http::Status;
use serde::Serialize;
use std::convert::{TryFrom, TryInto};

#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
pub struct Shop {
    pub slug: String,
    pub name: String,
    pub color: String,
    pub logo: String,
    #[serde(skip_serializing)]
    pub owner: String,
}

impl TryFrom<Row> for Shop {
    type Error = Error;
    fn try_from(row: Row) -> Result<Self> {
        Ok(Self {
            slug: row.try_get("slug")?,
            name: row.try_get("name")?,
            color: row.try_get("color")?,
            logo: row.try_get("logo")?,
            owner: row.try_get("owner")?,
        })
    }
}

impl Shop {
    pub async fn list(db: &Database) -> Result<Vec<Shop>> {
        db.run(move |db| {
            db.query(
                "SELECT *
                FROM shops",
                &[],
            )
        })
        .await?
        .into_iter()
        .map(Shop::try_from)
        .collect()
    }
    pub async fn read(db: &Database, slug: &str) -> Result<Shop> {
        let slug: String = slug.into();
        db.run(move |db| {
            db.query_one(
                "SELECT *
                FROM shops
                WHERE slug = $1",
                &[&slug],
            )
            .map_err(|e| {
                Error::builder_from(e)
                    .code(Status::NotFound)
                    .description("Loja não encontrada")
            })
        })
        .await?
        .try_into()
    }
    pub async fn list_from_user(db: &Database, user: &User) -> Result<Vec<Shop>> {
        let user = user.clone();
        db.run(move |db| {
            db.query(
                "SELECT *
                FROM shops
                WHERE owner = $1",
                &[&user.email],
            )
        })
        .await?
        .into_iter()
        .map(Shop::try_from)
        .collect()
    }
    pub async fn delete(&self, db: &Database) -> Result<()> {
        let shop = self.clone();
        db.run(move |db| {
            db.execute(
                "DELETE FROM shops
                WHERE slug = $1",
                &[&shop.slug],
            )
        })
        .await?;
        Ok(())
    }
    pub async fn update(&self, db: &Database, old_slug: &str) -> Result<()> {
        let old_slug: String = old_slug.into();
        let shop = self.clone();
        db.run(move |db| {
            db.execute(
                "UPDATE shops SET slug = $1, name = $2, color = $3, logo = $4, owner = $5
                WHERE slug = $6",
                &[
                    &shop.slug,
                    &shop.name,
                    &shop.color,
                    &shop.logo,
                    &shop.owner,
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
        let shop = self.clone();
        db.run(move |db| {
            db.execute(
                "INSERT INTO shops (slug, name, color, logo, owner) VALUES ($1, $2, $3, $4, $5)",
                &[&shop.slug, &shop.name, &shop.color, &shop.logo, &shop.owner],
            )
            .map_err(|e| {
                Error::builder_from(e)
                    .code(Status::BadRequest)
                    .description("Uma loja com esse identificador já existe")
            })
        })
        .await?;
        Ok(())
    }
}
