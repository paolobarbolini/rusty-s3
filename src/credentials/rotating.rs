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
