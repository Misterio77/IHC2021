use crate::schema::{User, UserToken};
use crate::{BodyResult, Database, Error, Result};

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, post};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}
#[post("/", data = "<body>")]
async fn login(db: Database, body: BodyResult<'_, LoginRequest>) -> Result<Json<User>> {
    let body = body?.into_inner();
    let mut user = User::read(&db, &body.email).await?;
    user.token = Some(User::generate_token()?);
    user.update(&db, &user.email).await?;
    if user.verify_password(&body.password) {
        Ok(Json(user))
    } else {
        Err(Error::builder()
            .code(Status::Unauthorized)
            .description("Senha incorreta")
            .build())
    }
}

#[delete("/")]
async fn logout(db: Database, token: Result<UserToken>) -> Result<()> {
    let mut user = User::read_from_token(&db, &token?).await?;
    user.token = None;
    user.update(&db, &user.email).await?;
    Ok(())
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![login, logout]
}
