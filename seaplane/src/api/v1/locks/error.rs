//! Errors that come from the Locks endpoints

use std::{error::Error, fmt};

use reqwest::{blocking::Response, StatusCode};
use serde::Deserialize;

use crate::error::Result;

#[derive(Clone, Debug, Deserialize)]
pub struct ErrorResponse {
    pub status: u16,
    pub title: String,
    pub detail: Option<String>,
}

pub fn map_error(resp: Response) -> Result<Response> {
    if let Err(source) = resp.error_for_status_ref() {
        let kind = source.status().into();
        return Err(LocksError {
            body: resp.json()?,
            source,
            kind,
        }
        .into());
    }
    Ok(resp)
}

#[derive(Debug)]
#[non_exhaustive]
pub struct LocksError {
    pub body: ErrorResponse,
    pub source: reqwest::Error,
    pub kind: LocksErrorKind,
}

impl fmt::Display for LocksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.body.title,
            if let Some(msg) = &self.body.detail {
                format!(" - {}", msg)
            } else {
                String::new()
            }
        )
    }
}

impl Error for LocksError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

impl PartialEq for LocksError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[non_exhaustive]
pub enum LocksErrorKind {
    /// HTTP Status Code that isn't implemented yet
    UnimplementedHttpStatus(StatusCode),
    /// HTTP 400 - Bad Request
    InvalidRequest,
    /// HTTP 401 - I don't know you
    NotLoggedIn,
    /// HTTP 404 - Not Found
    LockNotFound,
    /// HTTP 409 - Conflict
    LockAlreadyHeld,
    /// HTTP 500 - Internal
    InternalError,
    /// HTTP 503 - Service Unavailable
    ServiceUnavailable,
    /// Not an HTTP status error
    Unknown,
}

impl From<Option<StatusCode>> for LocksErrorKind {
    fn from(code: Option<StatusCode>) -> Self {
        use LocksErrorKind::*;
        match code {
            Some(StatusCode::BAD_REQUEST) => InvalidRequest,
            Some(StatusCode::UNAUTHORIZED) => NotLoggedIn,
            Some(StatusCode::NOT_FOUND) => LockNotFound,
            Some(StatusCode::CONFLICT) => LockAlreadyHeld,
            Some(StatusCode::INTERNAL_SERVER_ERROR) => InternalError,
            Some(StatusCode::SERVICE_UNAVAILABLE) => ServiceUnavailable,
            Some(code) => UnimplementedHttpStatus(code),
            None => Unknown,
        }
    }
}
