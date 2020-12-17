use std::env;
use std::fmt::{self, Debug, Formatter};

/// S3 credentials
#[derive(Clone, PartialEq, Eq)]
pub struct Credentials {
    key: String,
    secret: String,
}

impl Credentials {
    /// Construct a new `Credentials` using the provided key and secret
    #[inline]
    pub fn new(key: String, secret: String) -> Self {
        Self { key, secret }
    }

    /// Construct a new `Credentials` using AWS's default environment variables
    ///
    /// Reads the key from the `AWS_ACCESS_KEY_ID` environment variable and the secret
    /// from the `AWS_SECRET_ACCESS_KEY` environment variable.
    /// Returns `None` if either environment variables aren't set or they aren't valid utf-8.
    pub fn from_env() -> Option<Self> {
        let key = env::var("AWS_ACCESS_KEY_ID").ok()?;
        let secret = env::var("AWS_SECRET_ACCESS_KEY").ok()?;
        Some(Self::new(key, secret))
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
    }

    #[test]
    fn debug() {
        let credentials = Credentials::new("abcd".into(), "1234".into());
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

        env::remove_var("AWS_ACCESS_KEY_ID");
        env::remove_var("AWS_SECRET_ACCESS_KEY");

        assert!(Credentials::from_env().is_none());
    }
}
