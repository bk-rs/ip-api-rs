use core::fmt;

use http_api_client_endpoint::http::Error as HttpError;
use serde_json::Error as SerdeJsonError;
use url::ParseError as UrlParseError;

//
#[derive(Debug)]
pub enum EndpointError {
    MakeRequestUrlFailed(UrlParseError),
    MakeRequestFailed(HttpError),
    SerRequestBodyJsonFailed(SerdeJsonError),
    DeResponseBodyJsonFailed(SerdeJsonError),
}

impl fmt::Display for EndpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EndpointError {}
