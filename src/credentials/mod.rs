//! Credentials management types
//!
//! [`RotatingCredentials`] wraps [`Credentials`] and gives the ability to
//! rotate them at any point in the program, keeping all copies of the same
//! [`RotatingCredentials`] in sync with the latest version.
//!
//! [`Ec2SecurityCredentialsMetadataResponse`] parses the response from the
//! [EC2 metadata service](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/instancedata-data-retrieval.html),
//! which provides an endpoint for retrieving credentials using the permissions
//! for the [attached IAM roles](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/iam-roles-for-amazon-ec2.html).

use std::env;
use std::fmt::{self, Debug, Formatter};

#[allow(clippy::module_name_repetitions)]
pub use self::rotating::RotatingCredentials;
#[cfg(feature = "full")]
pub use self::serde::Ec2SecurityCredentialsMetadataResponse;
use zeroize::Zeroizing;

mod rotating;
#[cfg(feature = "full")]
mod serde;

/// S3 credentials
#[derive(Clone, PartialEq, Eq)]
pub struct Credentials {
    key: String,
    secret: Zeroizing<String>,
    token: Option<String>,
}

impl Credentials {
    /// Construct a new `Credentials` using the provided key and secret
    #[inline]
    pub fn new(key: impl Into<String>, secret: impl Into<String>) -> Self {
        Self::new_with_maybe_token(key.into(), secret.into(), None)
    }

    /// Construct a new `Credentials` using the provided key, secret and token
    #[inline]
    pub fn new_with_token(
        key: impl Into<String>,
        secret: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        Self::new_with_maybe_token(key.into(), secret.into(), Some(token.into()))
    }

    #[inline]
    pub(super) fn new_with_maybe_token(key: String, secret: String, token: Option<String>) -> Self {
        Self {
            key,
            secret: Zeroizing::new(secret),
            token,
        }
    }

    /// Construct a new `Credentials` using AWS's default environment variables
    ///
    /// Reads the key from the `AWS_ACCESS_KEY_ID` environment variable and the secret
    /// from the `AWS_SECRET_ACCESS_KEY` environment variable.
    /// If `AWS_SESSION_TOKEN` is set a token is also read.
    /// Returns `None` if either environment variables aren't set or they aren't valid utf-8.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let key = env::var("AWS_ACCESS_KEY_ID").ok()?;
        let secret = env::var("AWS_SECRET_ACCESS_KEY").ok()?;
        let token = env::var("AWS_SESSION_TOKEN").ok();
        Some(Self::new_with_maybe_token(key, secret, token))
    }

    /// Get the key of this `Credentials`
    #[inline]
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the secret of this `Credentials`
    #[inline]
    #[must_use]
    pub fn secret(&self) -> &str {
        &self.secret
    }

    /// Get the token of this `Credentials`, if present
    #[inline]
    #[must_use]
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }
}

impl Debug for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credentials")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn key_secret() {
        let credentials = Credentials::new("abcd", "1234");
        assert_eq!(credentials.key(), "abcd");
        assert_eq!(credentials.secret(), "1234");
        assert!(credentials.token().is_none());
    }

    #[test]
    fn key_secret_token() {
        let credentials = Credentials::new_with_token("abcd", "1234", "xyz");
        assert_eq!(credentials.key(), "abcd");
        assert_eq!(credentials.secret(), "1234");
        assert_eq!(credentials.token(), Some("xyz"));
    }

    #[test]
    fn debug() {
        let credentials = Credentials::new("abcd", "1234");
        let debug_output = format!("{credentials:?}");
        assert_eq!(debug_output, "Credentials { key: \"abcd\", .. }");
    }

    #[test]
    fn debug_token() {
        let credentials = Credentials::new_with_token("abcd", "1234", "xyz");
        let debug_output = format!("{credentials:?}");
        assert_eq!(debug_output, "Credentials { key: \"abcd\", .. }");
    }

    #[test]
    fn from_env() {
        env::set_var("AWS_ACCESS_KEY_ID", "key");
        env::set_var("AWS_SECRET_ACCESS_KEY", "secret");

        let credentials = Credentials::from_env().unwrap();
        assert_eq!(credentials.key(), "key");
        assert_eq!(credentials.secret(), "secret");
        assert!(credentials.token().is_none());

        env::remove_var("AWS_ACCESS_KEY_ID");
        env::remove_var("AWS_SECRET_ACCESS_KEY");

        assert!(Credentials::from_env().is_none());
    }
}
