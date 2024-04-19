use std::fmt;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use cdk::error::ErrorResponse;
use cdk::lightning_invoice::ParseOrSemanticError;

#[derive(Debug)]
pub enum EcashError {
    DecodeInvoice,
    StatusCode(StatusCode),
    _Ln(ln_rs::Error),
}

impl std::error::Error for EcashError {}

impl fmt::Display for EcashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DecodeInvoice => write!(f, "Failed to decode LN Invoice"),
            Self::StatusCode(code) => write!(f, "{}", code),
            Self::_Ln(code) => write!(f, "{}", code),
        }
    }
}

impl From<StatusCode> for EcashError {
    fn from(code: StatusCode) -> Self {
        Self::StatusCode(code)
    }
}

impl From<ParseOrSemanticError> for EcashError {
    fn from(_err: ParseOrSemanticError) -> Self {
        Self::DecodeInvoice
    }
}

impl From<url::ParseError> for EcashError {
    fn from(_err: url::ParseError) -> Self {
        Self::DecodeInvoice
    }
}

impl IntoResponse for EcashError {
    fn into_response(self) -> Response {
        match self {
            EcashError::DecodeInvoice => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            EcashError::StatusCode(code) => (code, "").into_response(),
            EcashError::_Ln(code) => {
                (StatusCode::INTERNAL_SERVER_ERROR, code.to_string()).into_response()
            }
        }
    }
}

pub fn into_response(error: cdk::mint::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json::<ErrorResponse>(error.into()),
    )
        .into_response()
}
