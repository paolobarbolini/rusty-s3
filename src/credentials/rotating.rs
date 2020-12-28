use std::cell::{Cell, Ref, RefCell};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;

use super::Credentials;

/// S3 credentials
pub struct RotatingCredentials {
    inner: Arc<RotatingCredentialsInner>,
}

struct RotatingCredentialsInner {
    creds1: RefCell<Credentials>,
    creds2: RefCell<Credentials>,
    in_use: Cell<InUse>,
}

#[derive(Copy, Clone)]
enum InUse {
    Creds1,
    Creds2,
}

impl RotatingCredentials {
    /// Construct a new `RotatingCredentials` using the provided key and secret
    pub fn new(key: String, secret: String, token: String) -> Self {
        Self {
            inner: Arc::new(RotatingCredentialsInner {
                creds1: RefCell::new(Credentials::new_with_token(key, secret, token)),
                creds2: RefCell::new(Credentials::empty()),
                in_use: Cell::new(InUse::Creds1),
            }),
        }
    }

    pub fn current_credentials(&self) -> Ref<'_, Credentials> {
        match self.inner.in_use.get() {
            InUse::Creds1 => self.inner.creds1.borrow(),
            InUse::Creds2 => self.inner.creds2.borrow(),
        }
    }

    pub fn update(&self, key: String, secret: String, token: String) {
        let credentials = Credentials::new_with_token(key, secret, token);

        match self.inner.in_use.get() {
            InUse::Creds1 => {
                self.inner.creds2.replace(credentials);
                self.inner.in_use.set(InUse::Creds2);
            }
            InUse::Creds2 => {
                self.inner.creds1.replace(credentials);
                self.inner.in_use.set(InUse::Creds1);
            }
        }
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
