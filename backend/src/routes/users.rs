use crate::schema::{User, UserToken};
use crate::{BodyResult, Database, Error, Result};
use futures::try_join;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put};
use serde::Deserialize;

#[get("/<email>")]
async fn read(db: Database, token: Result<UserToken>, email: String) -> Result<Json<User>> {
    let token = token?;
    let requester = db.run(move |db| User::from_token(db, token));
    let target = db.run(move |db| User::from_email(db, &email));

    let (requester, target) = try_join!(requester, target)?;
    if requester.email == target.email || requester.admin {
        Ok(Json(target))
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
async fn create(
    db: Database,
    body: BodyResult<'_, RegisterRequest>,
) -> Result<status::Created<Json<User>>> {
    let body = body?;
    let user = db
        .run(move |db| User::register(db, &body.email, &body.password, &body.name))
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
    admin: Option<bool>,
}
#[put("/<email>", data = "<body>")]
async fn update(
    db: Database,
    token: Result<UserToken>,
    body: BodyResult<'_, UpdateRequest>,
    email: String,
) -> Result<Json<User>> {
    let body = body?;
    let token = token?;
    let requester = db.run(move |db| User::from_token(db, token));
    let target = db.run(move |db| User::from_email(db, &email));

    let (requester, target) = try_join!(requester, target)?;
    // Apenas um administrador ou o próprio usuário podem mudar as informações
    if target.email == requester.email || requester.admin {
        let admin = body.admin.map(|request| request && requester.admin);
        let user = db.run(move |db| {
            target.modify(
                db,
                body.email.as_deref(),
                body.password.as_deref(),
                body.name.as_deref(),
                admin,
            )
        });
        Ok(Json(user.await?))
    } else {
        Err(Error::builder()
            .code(Status::Unauthorized)
            .description("Você não tem permissão para modificar esse usuário")
            .build())
    }
}

#[delete("/<email>")]
async fn delete(
    db: Database,
    token: Result<UserToken>,
    email: String,
) -> Result<status::NoContent> {
    let token = token?;
    let requester = db.run(move |db| User::from_token(db, token));
    let target = db.run(move |db| User::from_email(db, &email));

    let (requester, target) = try_join!(requester, target)?;
    // Apenas um administrador ou o próprio usuário podem mudar as informações
    if target.email == requester.email || requester.admin {
        db.run(move |db| target.delete(db)).await?;
        Ok(status::NoContent)
    } else {
        Err(Error::builder()
            .code(Status::Unauthorized)
            .description("Você não tem permissão para remover esse usuário")
            .build())
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![read, create, update, delete]
}
