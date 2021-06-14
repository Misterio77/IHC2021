use crate::schema::{User, UserToken};
use crate::{BodyResult, Database, Error, Result};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put};
use serde::Deserialize;

#[get("/<email>")]
async fn user(db: Database, token: Result<UserToken>, email: String) -> Result<Json<User>> {
    let user = db
        .run(move |db| -> Result<User> { User::from_token(db, token?) })
        .await?;
    if email == user.email {
        Ok(Json(user))
    } else {
        Err(Error::builder()
            .code(Status::Unauthorized)
            .description("Você não tem permissão para ver esse usuário")
            .build())
    }
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    name: String,
}

#[post("/", data = "<body>")]
async fn register(
    db: Database,
    body: BodyResult<'_, RegisterRequest>,
) -> Result<status::Created<Json<User>>> {
    let body = body?;
    let user = db
        .run(move |db| -> Result<User> {
            User::register(db, &body.email, &body.password, &body.name)
        })
        .await?;
    Ok(status::Created::new(format!(
        "https://cincobola.misterio.me/users/{}",
        user.email
    ))
    .body(Json(user)))
}

#[derive(Debug, Deserialize)]
struct UpdateRequest {
    email: Option<String>,
    password: Option<String>,
    name: Option<String>,
}
#[put("/<email>", data = "<body>")]
async fn update(
    db: Database,
    token: Result<UserToken>,
    body: BodyResult<'_, UpdateRequest>,
    email: String,
) -> Result<Json<User>> {
    let body = body?;
    let user = db
        .run(move |db| -> Result<User> {
            let user = User::from_token(db, token?)?;
            if email != user.email {
                return Err(Error::builder()
                    .code(Status::Unauthorized)
                    .description("Você não tem permissão para modificar esse usuário")
                    .build());
            }
            user.modify(
                db,
                body.email.as_deref(),
                body.password.as_deref(),
                body.name.as_deref(),
            )
        })
        .await?;
    Ok(Json(user))
}

#[delete("/<email>")]
async fn delete(
    db: Database,
    token: Result<UserToken>,
    email: String,
) -> Result<status::NoContent> {
    db.run(move |db| -> Result<()> {
        let user = User::from_token(db, token?)?;
        if email != user.email {
            return Err(Error::builder()
                .code(Status::Unauthorized)
                .description("Você não tem permissão para remover esse usuário")
                .build());
        }
        user.delete(db)
    })
    .await?;
    Ok(status::NoContent)
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![user, register, update, delete]
}
