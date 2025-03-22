use std::fmt;

#[derive(Debug)]
pub enum ServiceError {
    AuthenticationError(String),
    BadRequest(String),
    ExternalService(String),
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
            ServiceError::ExternalService(msg) => write!(f, "ExternalService : {}", msg),
            ServiceError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ServiceError::DatabaseError(msg) => write!(f, "Not Found: {}", msg),
            ServiceError::InternalServerError => write!(f, "Internal Server Error"),
            ServiceError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}
