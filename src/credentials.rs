use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq, Eq)]
pub struct Credentials {
    key: String,
    secret: String,
}

impl Credentials {
    pub fn new(key: String, secret: String) -> Self {
        Self { key, secret }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

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
}
