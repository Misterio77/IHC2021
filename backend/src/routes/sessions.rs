use crate::schema::{User, UserSession, UserToken};
use crate::{BodyResult, Database, Result};

use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{delete, get, post};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[post("/", data = "<body>")]
async fn login(db: Database, body: BodyResult<'_, LoginRequest>) -> Result<Json<UserToken>> {
    let body = body?;
    let token = db
        .run(move |db| -> Result<UserToken> {
            let user = User::from_credentials(db, &body.email, &body.password)?;
            user.create_token(db)
        })
        .await?;
    Ok(Json(token))
}

#[get("/")]
async fn sessions(db: Database, token: Result<UserToken>) -> Result<Json<Vec<UserSession>>> {
    let token = token?;
    let sessions = db
        .run(move |db| -> Result<Vec<UserSession>> {
            let user = User::from_token(db, token)?;
            user.list_sessions(db)
        })
        .await?;
    Ok(Json(sessions))
}

#[delete("/")]
async fn delete_sessions(db: Database, token: Result<UserToken>) -> Result<status::NoContent> {
    let token = token?;
    db.run(move |db| -> Result<()> {
        let user = User::from_token(db, token)?;
        user.delete_session(db, None)
    })
    .await?;
    Ok(status::NoContent)
}

#[delete("/<id>")]
async fn delete_session(
    db: Database,
    id: i32,
    token: Result<UserToken>,
) -> Result<status::NoContent> {
    let token = token?;
    db.run(move |db| -> Result<()> {
        let user = User::from_token(db, token)?;
        user.delete_session(db, Some(id))
    })
    .await?;
    Ok(status::NoContent)
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![login, sessions, delete_sessions, delete_session]
}
