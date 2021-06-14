use crate::schema::{User, UserToken};
use crate::{BodyResult, Database, Error, Result};
use futures::try_join;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put};
use serde::Deserialize;

#[get("/")]
async fn list(db: Database, token: Result<UserToken>) -> Result<Json<Vec<User>>> {
    let token = token?;
    let requester = User::read_from_token(&db, &token).await?;
    if requester.admin {
        let users = User::list(&db).await?;
        Ok(Json(users))
    } else {
        Err(Error::builder()
            .code(Status::Forbidden)
            .description("Você não tem permissão para listar os usuários")
            .build())
    }
}
#[get("/<email>")]
async fn read(db: Database, token: Result<UserToken>, email: String) -> Result<Json<User>> {
    let token = token?;
    let requester = User::read_from_token(&db, &token);
    let target = User::read(&db, &email);

    let (requester, target) = try_join!(requester, target)?;
    if requester.email != target.email && !requester.admin {
        return Err(Error::builder()
            .code(Status::Forbidden)
            .description("Você não tem permissão para ver esse usuário")
            .build());
    }
    Ok(Json(target))
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
    let body = body?.into_inner();
    let user = User {
        email: body.email,
        password: User::hash_password(&body.password)?,
        name: body.name,
        token: Some(User::generate_token()?),
        admin: false,
    };

    user.create(&db).await?;

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
    let body = body?.into_inner();
    let token = token?;
    let requester = User::read_from_token(&db, &token);
    let target = User::read(&db, &email);

    let (requester, mut target) = try_join!(requester, target)?;
    let old_email = target.email.clone();

    // Apenas um administrador ou o próprio usuário podem mudar as informações
    if target.email != requester.email && !requester.admin {
        return Err(Error::builder()
            .code(Status::Forbidden)
            .description("Você não tem permissão para modificar esse usuário")
            .build());
    }

    if let Some(x) = body.email {
        target.email = x;
    }
    if let Some(x) = body.password {
        target.password = User::hash_password(&x)?;
    }
    if let Some(x) = body.name {
        target.name = x;
    }
    if let Some(x) = body.admin {
        target.admin = x && requester.admin;
    }
    target.update(&db, &old_email).await?;
    Ok(Json(target))
}

#[delete("/<email>")]
async fn delete(
    db: Database,
    token: Result<UserToken>,
    email: String,
) -> Result<status::NoContent> {
    let token = token?;
    let requester = User::read_from_token(&db, &token);
    let target = User::read(&db, &email);

    let (requester, target) = try_join!(requester, target)?;
    // Apenas um administrador ou o próprio usuário podem apagar
    if target.email != requester.email && !requester.admin {
        return Err(Error::builder()
            .code(Status::Forbidden)
            .description("Você não tem permissão para remover esse usuário")
            .build());
    }
    target.delete(&db).await?;
    Ok(status::NoContent)
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![list, read, create, update, delete]
}
