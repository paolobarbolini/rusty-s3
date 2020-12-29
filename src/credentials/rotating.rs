use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::{Arc, RwLock};

use super::Credentials;

/// Credentials that can be rotated
pub struct RotatingCredentials {
    inner: Arc<RwLock<Arc<Credentials>>>,
}

impl RotatingCredentials {
    /// Construct a new `RotatingCredentials` using the provided key and secret
    pub fn new(key: String, secret: String, token: String) -> Self {
        let credentials = Credentials::new_with_token(key, secret, token);

        Self {
            inner: Arc::new(RwLock::new(Arc::new(credentials))),
        }
    }

    pub fn current_credentials(&self) -> Arc<Credentials> {
        let lock = self.inner.read().expect("can't be poisoned");
        Arc::clone(&lock)
    }

    pub fn update(&self, key: String, secret: String, token: String) {
        let credentials = Credentials::new_with_token(key, secret, token);

        let mut lock = self.inner.write().expect("can't be poisoned");
        match Arc::get_mut(&mut lock) {
            Some(arc) => *arc = credentials,
            None => *lock = Arc::new(credentials),
        };
    }
}

impl Debug for RotatingCredentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let current = self.current_credentials();
        Debug::fmt(&*current, f)
    }
}

impl Clone for RotatingCredentials {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.inner = Arc::clone(&source.inner)
    }
}

impl PartialEq for RotatingCredentials {
    fn eq(&self, other: &RotatingCredentials) -> bool {
        let current1 = self.current_credentials();
        let current2 = other.current_credentials();
        *current1 == *current2
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn rotate() {
        let credentials = RotatingCredentials::new("abcd".into(), "1234".into(), "xyz".into());

        let current = credentials.current_credentials();
        assert_eq!(current.key(), "abcd");
        assert_eq!(current.secret(), "1234");
        assert_eq!(current.token(), Some("xyz"));
        drop(current);

        credentials.update("1234".into(), "5678".into(), "9012".into());

        let current = credentials.current_credentials();
        assert_eq!(current.key(), "1234");
        assert_eq!(current.secret(), "5678");
        assert_eq!(current.token(), Some("9012"));
        drop(current);

        credentials.update("dcba".into(), "4321".into(), "yxz".into());

        let current = credentials.current_credentials();
        assert_eq!(current.key(), "dcba");
        assert_eq!(current.secret(), "4321");
        assert_eq!(current.token(), Some("yxz"));
        drop(current);
    }

    #[test]
    fn rotate_cloned() {
        let credentials = RotatingCredentials::new("abcd".into(), "1234".into(), "xyz".into());

        let current = credentials.current_credentials();
        assert_eq!(current.key(), "abcd");
        assert_eq!(current.secret(), "1234");
        assert_eq!(current.token(), Some("xyz"));
        drop(current);

        let credentials2 = credentials.clone();

        credentials.update("1234".into(), "5678".into(), "9012".into());

        let current = credentials2.current_credentials();
        assert_eq!(current.key(), "1234");
        assert_eq!(current.secret(), "5678");
        assert_eq!(current.token(), Some("9012"));
        drop(current);

        assert_eq!(credentials, credentials2);

        credentials.update("dcba".into(), "4321".into(), "yxz".into());

        let current = credentials.current_credentials();
        assert_eq!(current.key(), "dcba");
        assert_eq!(current.secret(), "4321");
        assert_eq!(current.token(), Some("yxz"));
        drop(current);

        assert_eq!(credentials, credentials2);
    }

    #[test]
    fn debug() {
        let credentials = RotatingCredentials::new("abcd".into(), "1234".into(), "xyz".into());
        let debug_output = format!("{:?}", credentials);
        assert_eq!(debug_output, "Credentials { key: \"abcd\" }");
    }
}
