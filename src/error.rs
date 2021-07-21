use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::io;

use rocket::http::Status;
use serde::ser::{Serialize, SerializeStruct, Serializer};

pub struct ErrorBuilder {
    inner: Error,
}

impl ErrorBuilder {
    pub fn source(self, source: Box<dyn StdError + Sync + Send>) -> ErrorBuilder {
        ErrorBuilder {
            inner: Error {
                code: self.inner.code,
                description: self.inner.description,
                source: Some(source),
            },
        }
    }
    pub fn code(self, code: Status) -> ErrorBuilder {
        ErrorBuilder {
            inner: Error {
                code,
                source: self.inner.source,
                description: self.inner.description,
            },
        }
    }
    pub fn description(self, description: &str) -> ErrorBuilder {
        ErrorBuilder {
            inner: Error {
                code: self.inner.code,
                source: self.inner.source,
                description: Some(String::from(description)),
            },
        }
    }
    pub fn missing_header(self, header: &str) -> ErrorBuilder {
        self.code(Status::Unauthorized)
            .description(&format!("Esse request deve conter o header '{}'", header))
    }
    pub fn build(self) -> Error {
        self.inner
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    code: Status,
    source: Option<Box<dyn StdError + Sync + Send>>,
    description: Option<String>,
}

impl Error {
    pub fn builder() -> ErrorBuilder {
        Error::default().edit()
    }
    pub fn edit(self) -> ErrorBuilder {
        ErrorBuilder { inner: self }
    }
    pub fn builder_from<T: Into<Error>>(source: T) -> ErrorBuilder {
        let error = source.into();
        error.edit()
    }
}

impl Default for Error {
    fn default() -> Self {
        Error {
            code: Status::InternalServerError,
            source: None,
            description: None,
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|s| &**s as _)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code.reason_lossy())?;
        if let Some(description) = &self.description {
            write!(f, " ({})", description)?;
        };
        if let Some(source) = &self.source {
            write!(f, ": {}", source)?;
        };
        Ok(())
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Error", 2)?;
        state.serialize_field("code", &format!("{}", &self.code))?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field(
            "reason",
            &self
                .source
                .as_ref()
                .map(|s| format!("{:?}", s).replace("\"", "'")),
        )?;
        state.end()
    }
}

impl From<ErrorBuilder> for Error {
    fn from(f: ErrorBuilder) -> Error {
        f.build()
    }
}

impl<'r> rocket::response::Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r rocket::request::Request<'_>) -> rocket::response::Result<'static> {
        let mut response_object = HashMap::new();
        response_object.insert("error", &self);
        let json = serde_json::to_string(&response_object).unwrap_or_else(|_| "".to_string());
        rocket::Response::build()
            .status(self.code)
            .header(rocket::http::ContentType::JSON)
            .sized_body(None, io::Cursor::new(json))
            .ok()
    }
}

impl From<rocket::error::Error> for Error {
    fn from(e: rocket::error::Error) -> Self {
        Error::builder()
            .code(Status::ServiceUnavailable)
            .source(Box::new(e))
            .description("Não foi possível iniciar o servidor")
            .build()
    }
}

impl From<postgres::Error> for Error {
    fn from(e: postgres::Error) -> Self {
        Error::builder()
            .description("Não foi possível completar operação")
            .source(Box::new(e))
            .build()
    }
}

impl From<argon2::Error> for Error {
    fn from(e: argon2::Error) -> Self {
        Error::builder()
            .source(Box::new(e))
            .description("Não foi possível gerar hash")
            .build()
    }
}

impl From<rocket::serde::json::Error<'_>> for Error {
    fn from(e: rocket::serde::json::Error) -> Self {
        let error: Box<dyn StdError + Sync + Send> = match e {
            rocket::serde::json::Error::Io(error) => Box::new(error),
            rocket::serde::json::Error::Parse(_, error) => Box::new(error),
        };
        Error::builder()
            .source(error)
            .description("O JSON da entrada é inválido para essa rota")
            .code(Status::BadRequest)
            .build()
    }
}
