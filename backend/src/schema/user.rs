use crate::{Error, Result};
use chrono::{DateTime, Utc};
use postgres::Row;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rocket::http::Status;
use rocket::request;
use serde::Serialize;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Serialize)]
pub struct UserToken(String);

impl From<UserToken> for String {
    fn from(f: UserToken) -> String {
        f.0
    }
}

#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for UserToken {
    /// Erro a ser retornado em caso de falha
    type Error = Error;
    /// Tentar extrair um [`UserToken`] do request
    async fn from_request(req: &'r request::Request<'_>) -> request::Outcome<Self, Error> {
        let token = req.headers().get("Authentication").next();
        match token {
            None => request::Outcome::Failure((
                Status::Unauthorized,
                Error::builder().missing_header("Authentication").build(),
            )),
            Some(token) => request::Outcome::Success(UserToken(token.to_string())),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserSession {
    pub id: i32,
    pub created: DateTime<Utc>,
    pub used: Option<DateTime<Utc>>,
}

impl TryFrom<Row> for UserSession {
    type Error = Error;
    fn try_from(row: Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            created: row.try_get("created")?,
            used: row.try_get("used")?,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct User {
    pub email: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub password: String,
}

impl TryFrom<Row> for User {
    type Error = Error;
    fn try_from(row: Row) -> Result<Self> {
        Ok(Self {
            email: row.try_get("email")?,
            name: row.try_get("name")?,
            password: row.try_get("password")?,
        })
    }
}

impl User {
    /// Dado token, busca um usuário na db
    pub fn from_token(db: &mut postgres::Client, token: UserToken) -> Result<User> {
        let row = db
            .query_one(
                "SELECT email, name, password, sessions.id AS session_id
                FROM users
                INNER JOIN sessions
                ON sessions.user_email = users.email
                WHERE sessions.token = $1",
                &[&token.0],
            )
            .map_err(|e| {
                Error::builder_from(e)
                    .code(Status::Unauthorized)
                    .description("Sessão inválida")
            })?;
        let session_id: i32 = row.try_get("session_id")?;
        db.execute("UPDATE sessions SET used=now() WHERE id=$1", &[&session_id])?;
        row.try_into()
    }
    /// Utilizando credenciais (email e senha), busca um usuário na database, verificando a senha
    pub fn from_credentials(
        db: &mut postgres::Client,
        email: &str,
        password: &str,
    ) -> Result<Self> {
        let user: User = db
            .query_one(
                "SELECT email, name, password
                FROM users
                WHERE email = $1",
                &[&email],
            )
            .map_err(|e| {
                Error::builder_from(e)
                    .code(Status::Unauthorized)
                    .description("Esse usuário não está cadastrado")
            })?
            .try_into()?;
        if user.verify_password(password) {
            Ok(user)
        } else {
            Err(Error::builder()
                .code(Status::Unauthorized)
                .description("Senha incorreta")
                .build())
        }
    }
    /// Cria um novo token para o usuário
    pub fn create_token(&self, db: &mut postgres::Client) -> Result<UserToken> {
        let token = thread_rng()
            .sample_iter(Alphanumeric)
            .take(128)
            .map(char::from)
            .collect();
        db.execute(
            "INSERT INTO sessions (token, user_email)
            VALUES ($1, $2)",
            &[&token, &self.email],
        )?;
        Ok(UserToken(token))
    }
    /// Lista as sessões ativas do usuário
    pub fn list_sessions(&self, db: &mut postgres::Client) -> Result<Vec<UserSession>> {
        db.query(
            "SELECT id, created, used
            FROM sessions
            WHERE user_email = $1",
            &[&self.email],
        )?
        .into_iter()
        .map(UserSession::try_from)
        .collect()
    }
    /// Revoga uma sessão específica, ou todas, em caso de None
    pub fn delete_session(&self, db: &mut postgres::Client, id: Option<i32>) -> Result<()> {
        match id {
            Some(id) => db.execute(
                "DELETE FROM sessions
                WHERE user_email = $1 AND id = $2",
                &[&self.email, &id],
            )?,
            None => db.execute(
                "DELETE FROM sessions
                WHERE user_email = $1",
                &[&self.email],
            )?,
        };
        Ok(())
    }
    /// Dado uma senha em cleartext, verifica se ela bate com o hash armazenado
    pub fn verify_password(&self, password: &str) -> bool {
        argon2::verify_encoded(&self.password, password.as_bytes()).unwrap_or(false)
    }
    /// Modifica informações (requer confirmação da senha)
    pub fn modify(
        self,
        db: &mut postgres::Client,
        current_password: &str,
        new_email: Option<&str>,
        new_password: Option<&str>,
        new_name: Option<&str>,
    ) -> Result<User> {
        let mut user = self;
        let old_email = user.email.clone();
        if !user.verify_password(current_password) {
            return Err(Error::builder()
                .code(Status::Unauthorized)
                .description("Senha incorreta")
                .build());
        }
        if let Some(new_email) = new_email {
            user.email = new_email.into();
        }
        if let Some(new_password) = new_password {
            user.password = hash_password(new_password)?;
        }
        if let Some(new_name) = new_name {
            user.name = new_name.into();
        }

        db.execute(
            "UPDATE users SET email = $1, password = $2, name = $3
            WHERE email = $4",
            &[&user.email, &user.password, &user.name, &old_email],
        )
        .map_err(|e| {
            Error::builder_from(e)
                .code(Status::InternalServerError)
                .description("Não foi possível atualizar informações")
        })?;
        Ok(user)
    }
    /// Remove o usuário (requer confirmação da senha)
    pub fn delete(self, db: &mut postgres::Client, current_password: &str) -> Result<()> {
        if !self.verify_password(current_password) {
            return Err(Error::builder()
                .code(Status::Unauthorized)
                .description("Senha incorreta")
                .build());
        }
        db.execute(
            "DELETE FROM users
            WHERE email = $1",
            &[&self.email],
        )?;
        Ok(())
    }
    /// Utilizando os dados, registra um novo usuário
    pub fn register(
        db: &mut postgres::Client,
        email: &str,
        password: &str,
        name: &str,
    ) -> Result<Self> {
        // Instanciar usuário
        let user = User {
            email: email.into(),
            password: hash_password(password)?,
            name: name.into(),
        };
        // Guardar na database
        db.execute(
            "INSERT INTO users VALUES ($1, $2, $3)",
            &[&user.email, &user.password, &user.name],
        )
        .map_err(|e| {
            // Caso não dê, já existe um registro com esse email (PK) lá
            Error::builder_from(e)
                .code(Status::BadRequest)
                .description("O email especificado já está registrado")
        })?;
        Ok(user)
    }
}
fn hash_password(password: &str) -> Result<String> {
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
