use std::{error, fmt, io::Error as IoError};

use reqwest::{Error as HttpError, StatusCode};
use serde_json::Error as SerdeError;
use serde_urlencoded::ser::Error as UrlEncodedError;
#[cfg(feature = "toml")]
use tomlcrate::de::Error as TomlDeError;
#[cfg(feature = "toml")]
use tomlcrate::ser::Error as TomlSerError;
use url::ParseError as UrlError;

/// Convience type over `std::result::Result` with `Error` as the error type.
pub type Result<T> = ::std::result::Result<T, Error>;

/// enum of possible errors encountered using the mastodon API.
#[derive(Debug)]
pub enum Error {
    /// Error from the Mastodon API. This typically means something went
    /// wrong with your authentication or data.
    Api(ApiError),
    /// Error deserialising to json. Typically represents a breaking change in
    /// the Mastodon API
    Serde(SerdeError),
    /// Error serializing to url-encoded string
    UrlEncoded(UrlEncodedError),
    /// Error encountered in the HTTP backend while requesting a route.
    Http(HttpError),
    /// Wrapper around the `std::io::Error` struct.
    Io(IoError),
    /// Wrapper around the `url::ParseError` struct.
    Url(UrlError),
    /// Missing Client Id.
    ClientIdRequired,
    /// Missing Client Secret.
    ClientSecretRequired,
    /// Missing Access Token.
    AccessTokenRequired,
    /// Generic client error.
    Client(StatusCode),
    /// Generic server error.
    Server(StatusCode),
    /// MastodonBuilder error.
    DataMissing,
    /// AppBuilder error
    MissingField(&'static str),
    #[cfg(feature = "toml")]
    /// Error serializing to toml
    TomlSer(TomlSerError),
    #[cfg(feature = "toml")]
    /// Error deserializing from toml
    TomlDe(TomlDeError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Api(ref e) => e
                .error_description
                .as_ref()
                .map(|i| &**i)
                .or(e.error.as_ref().map(|i| &**i))
                .unwrap_or("Unknown API Error"),
            Error::Serde(ref e) => e.description(),
            Error::UrlEncoded(ref e) => e.description(),
            Error::Http(ref e) => e.description(),
            Error::Io(ref e) => e.description(),
            Error::Url(ref e) => e.description(),
            Error::Client(ref status) | Error::Server(ref status) => {
                status.canonical_reason().unwrap_or("Unknown Status code")
            },
            Error::ClientIdRequired => "ClientIdRequired",
            Error::ClientSecretRequired => "ClientSecretRequired",
            Error::AccessTokenRequired => "AccessTokenRequired",
            Error::DataMissing => "DataMissing",
            Error::MissingField(_) => "MissingField",
            #[cfg(feature = "toml")]
            Error::TomlSer(ref e) => e.description(),
            #[cfg(feature = "toml")]
            Error::TomlDe(ref e) => e.description(),
        }
    }
}

/// Error returned from the Mastodon API.
#[derive(Clone, Debug, Deserialize)]
pub struct ApiError {
    /// The type of error.
    pub error: Option<String>,
    /// The description of the error.
    pub error_description: Option<String>,
}

macro_rules! from {
    ($($(#[$met:meta])* $typ:ident, $variant:ident,)*) => {
        $(
            $(#[$met])*
            impl From<$typ> for Error {
                fn from(from: $typ) -> Self {
                    use Error::*;
                    $variant(from)
                }
            }
        )*
    }
}

from! {
    HttpError, Http,
    IoError, Io,
    SerdeError, Serde,
    UrlEncodedError, UrlEncoded,
    UrlError, Url,
    ApiError, Api,
    #[cfg(feature = "toml")] TomlSerError, TomlSer,
    #[cfg(feature = "toml")] TomlDeError, TomlDe,
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest;
    use serde_json;
    use serde_urlencoded;
    use std::io;

    macro_rules! assert_is {
        ($err:ident, $variant:pat) => {
            assert!(match $err {
                $variant => true,
                _ => false,
            });
        };
    }

    #[test]
    fn from_http_error() {
        let err: HttpError = reqwest::get("not an actual URL").unwrap_err();
        let err: Error = Error::from(err);
        assert_is!(err, Error::Http(..));
    }

    #[test]
    fn from_io_error() {
        let err: IoError = io::Error::new(io::ErrorKind::Other, "other error");
        let err: Error = Error::from(err);
        assert_is!(err, Error::Io(..));
    }

    #[test]
    fn from_serde_error() {
        let err: SerdeError = serde_json::from_str::<()>("not valid json").unwrap_err();
        let err: Error = Error::from(err);
        assert_is!(err, Error::Serde(..));
    }

    #[test]
    fn from_url_encoded_error() {
        let err: UrlEncodedError = serde_urlencoded::ser::Error::Custom("error".into());
        let err: Error = Error::from(err);
        assert_is!(err, Error::UrlEncoded(..));
    }

    #[test]
    fn from_url_error() {
        let err: UrlError = UrlError::EmptyHost;
        let err: Error = Error::from(err);
        assert_is!(err, Error::Url(..));
    }

    #[test]
    fn from_api_error() {
        let err: ApiError = ApiError {
            error: None,
            error_description: None,
        };
        let err: Error = Error::from(err);
        assert_is!(err, Error::Api(..));
    }

    #[cfg(feature = "toml")]
    #[test]
    fn from_toml_ser_error() {
        let err: TomlSerError = TomlSerError::DateInvalid;
        let err: Error = Error::from(err);
        assert_is!(err, Error::TomlSer(..));
    }

    #[cfg(feature = "toml")]
    #[test]
    fn from_toml_de_error() {
        use tomlcrate;
        let err: TomlDeError = tomlcrate::from_str::<()>("not valid toml").unwrap_err();
        let err: Error = Error::from(err);
        assert_is!(err, Error::TomlDe(..));
    }
}
