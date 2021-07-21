use crate::{Database, Error, Result};
use postgres::Row;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rocket::http::Status;
use rocket::request;
use serde::Serialize;
use std::convert::{TryFrom, TryInto};

#[derive(Clone, Debug, Serialize)]
pub struct UserToken {
    token: String,
}

impl From<UserToken> for String {
    fn from(f: UserToken) -> String {
        f.token
    }
}

#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for UserToken {
    type Error = Error;
    async fn from_request(req: &'r request::Request<'_>) -> request::Outcome<Self, Error> {
        let token = req.headers().get("Authentication").next();
        match token {
            None => request::Outcome::Failure((
                Status::Unauthorized,
                Error::builder().missing_header("Authentication").build(),
            )),
            Some(token) => request::Outcome::Success(UserToken {
                token: token.into(),
            }),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize)]
pub struct User {
    pub email: String,
    pub name: String,
    pub admin: bool,
    #[serde(skip_serializing)]
    pub password: String,
    pub token: Option<String>,
}

impl TryFrom<Row> for User {
    type Error = Error;
    fn try_from(row: Row) -> Result<Self> {
        Ok(Self {
            email: row.try_get("email")?,
            name: row.try_get("name")?,
            admin: row.try_get("admin")?,
            password: row.try_get("password")?,
            token: row.try_get("token")?,
        })
    }
}

impl User {
    /// Lê um usuário da database, dado email
    pub async fn read(db: &Database, email: &str) -> Result<User> {
        let email: String = email.into();
        db.run(move |db| {
            db.query_one(
                "SELECT email, name, password, admin, token
                FROM users
                WHERE email = $1",
                &[&email],
            )
            .map_err(|e| {
                Error::builder_from(e)
                    .code(Status::NotFound)
                    .description("Usuário não encontrado")
            })
        })
        .await?
        .try_into()
    }
    /// Dado token, busca um usuário na db
    pub async fn read_from_token(db: &Database, token: &UserToken) -> Result<User> {
        let token = token.clone();
        db.run(move |db| {
            db.query_one(
                "SELECT email, name, password, admin, token
                    FROM users
                    WHERE token = $1",
                &[&token.token],
            )
            .map_err(|e| {
                Error::builder_from(e)
                    .code(Status::Unauthorized)
                    .description("Sessão inválida")
            })
        })
        .await?
        .try_into()
    }
    /// Lista todos os usuários
    pub async fn list(db: &Database) -> Result<Vec<User>> {
        db.run(move |db| {
            db.query(
                "SELECT email, name, password, admin, token
                FROM users",
                &[],
            )
        })
        .await?
        .into_iter()
        .map(User::try_from)
        .collect()
    }
    /// Modifica informações
    pub async fn update(&self, db: &Database, old_email: &str) -> Result<()> {
        let old_email: String = old_email.into();
        let user = self.clone();
        db.run(move |db| {
            db.execute(
                "UPDATE users SET email = $1, password = $2, name = $3, admin = $4, token = $5
                WHERE email = $6",
                &[
                    &user.email,
                    &user.password,
                    &user.name,
                    &user.admin,
                    &user.token,
                    &old_email,
                ],
            )
            .map_err(|e| {
                Error::builder_from(e).description("Não foi possível atualizar informações")
            })
        })
        .await?;
        Ok(())
    }
    /// Remove o usuário
    pub async fn delete(&self, db: &Database) -> Result<()> {
        let user = self.clone();
        db.run(move |db| {
            db.execute(
                "DELETE FROM users
                WHERE email = $1",
                &[&user.email],
            )
        })
        .await?;
        Ok(())
    }
    /// Utilizando os dados, registra um novo usuário
    pub async fn create(&self, db: &Database) -> Result<()> {
        let user = self.clone();
        db.run(move |db| {
            db.execute(
                "INSERT INTO users (email, password, name, admin, token) VALUES ($1, $2, $3, $4, $5)",
                &[
                    &user.email,
                    &user.password,
                    &user.name,
                    &user.admin,
                    &user.token,
                ],
            )
            .map_err(|e| {
                // Caso não dê, já existe um registro com esse email (PK) lá
                Error::builder_from(e)
                    .code(Status::BadRequest)
                    .description("O email especificado já está registrado")
            })
        }).await?;
        Ok(())
    }
    /// Dado uma senha em cleartext, verifica se ela bate com o hash armazenado
    pub fn verify_password(&self, password: &str) -> bool {
        argon2::verify_encoded(&self.password, password.as_bytes()).unwrap_or(false)
    }
    /// Gera um novo token de autenticação
    pub fn generate_token() -> Result<String> {
        Ok(thread_rng()
            .sample_iter(Alphanumeric)
            .take(128)
            .map(char::from)
            .collect())
    }
    /// Cria uma hash (com sal) de uma dada senha
    pub fn hash_password(password: &str) -> Result<String> {
        // Gerar sal aleatório
        // Um sal é aleatório, guardado em cleartext, e usado ao hashear uma senha.
        // O propósito é fazer que senhas iguais gerem hashes diferentes. Evitando ataques com
        // rainbow table (hashs pré-calculados) e hashes iguais na database.
        let mut salt = [0u8; 16];
        thread_rng().fill(&mut salt);
        // Criar o hash
        let hashed_password =
            argon2::hash_encoded(password.as_bytes(), &salt, &argon2::Config::default())?;
        Ok(hashed_password)
    }
}
