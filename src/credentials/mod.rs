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

pub use self::rotating::RotatingCredentials;
pub use self::serde::Ec2SecurityCredentialsMetadataResponse;

mod rotating;
mod serde;

/// S3 credentials
#[derive(Clone, PartialEq, Eq)]
pub struct Credentials {
    key: String,
    secret: String,
    token: Option<String>,
}

impl Credentials {
    /// Construct a new `Credentials` using the provided key and secret
    #[inline]
    pub fn new(key: String, secret: String) -> Self {
        Self::new_(key, secret, None)
    }

    /// Construct a new `Credentials` using the provided key, secret and token
    ///
    /// For backwards compatibility this method was named `new_`, and will replace
    /// the current `new` implementation in the 0.2.0 release.
    #[inline]
    pub fn new_(key: String, secret: String, token: Option<String>) -> Self {
        Self { key, secret, token }
    }

    /// Construct a new `Credentials` using AWS's default environment variables
    ///
    /// Reads the key from the `AWS_ACCESS_KEY_ID` environment variable and the secret
    /// from the `AWS_SECRET_ACCESS_KEY` environment variable.
    /// If `AWS_SESSION_TOKEN` is set a token is also read.
    /// Returns `None` if either environment variables aren't set or they aren't valid utf-8.
    pub fn from_env() -> Option<Self> {
        let key = env::var("AWS_ACCESS_KEY_ID").ok()?;
        let secret = env::var("AWS_SECRET_ACCESS_KEY").ok()?;
        let token = env::var("AWS_SESSION_TOKEN").ok();
        Some(Self::new_(key, secret, token))
    }

    /// Get the key of this `Credentials`
    #[inline]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the secret of this `Credentials`
    #[inline]
    pub fn secret(&self) -> &str {
        &self.secret
    }

    /// Get the token of this `Credentials`, if present
    #[inline]
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }
}

impl Debug for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credentials")
            .field("key", &self.key)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn key_secret() {
        let credentials = Credentials::new("abcd".into(), "1234".into());
        assert_eq!(credentials.key(), "abcd");
        assert_eq!(credentials.secret(), "1234");
        assert!(credentials.token().is_none());
    }

    #[test]
    fn key_secret_token() {
        let credentials = Credentials::new_("abcd".into(), "1234".into(), Some("xyz".into()));
        assert_eq!(credentials.key(), "abcd");
        assert_eq!(credentials.secret(), "1234");
        assert_eq!(credentials.token(), Some("xyz"));
    }

    #[test]
    fn debug() {
        let credentials = Credentials::new("abcd".into(), "1234".into());
        let debug_output = format!("{:?}", credentials);
        assert_eq!(debug_output, "Credentials { key: \"abcd\" }");
    }

    #[test]
    fn debug_token() {
        let credentials = Credentials::new_("abcd".into(), "1234".into(), Some("xyz".into()));
        let debug_output = format!("{:?}", credentials);
        assert_eq!(debug_output, "Credentials { key: \"abcd\" }");
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
