use std::fmt;

#[derive(Debug)]
pub enum ServiceError {
    AuthenticationError(String),
    BadRequest(String),
    NotFound(String),
    DatabaseError(String),
    InternalServerError,
    Unauthorized(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::AuthenticationError(msg) => write!(f, "Authentication Error: {}", msg),
            ServiceError::BadRequest(msg) => write!(f, "BadRequest : {}", msg),
            ServiceError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ServiceError::DatabaseError(msg) => write!(f, "Not Found: {}", msg),
            ServiceError::InternalServerError => write!(f, "Internal Server Error"),
            ServiceError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}

impl From<diesel::result::Error> for ServiceError {
    fn from(error: diesel::result::Error) -> Self {
        match error {
            diesel::result::Error::NotFound => ServiceError::NotFound("记录未找到".into()),
            _ => ServiceError::InternalServerError,
        }
    }
}