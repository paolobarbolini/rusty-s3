use std::fmt::{self, Debug, Formatter};
use std::mem;

use jiff::Timestamp;
use serde::Deserialize;
use zeroize::Zeroize as _;

use super::{Credentials, RotatingCredentials};

/// Parser for responses received from the EC2 security-credentials metadata service.
#[derive(Clone, Deserialize)]
pub struct Ec2SecurityCredentialsMetadataResponse {
    #[serde(rename = "AccessKeyId")]
    key: String,
    #[serde(rename = "SecretAccessKey")]
    secret: String,
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "Expiration")]
    expiration: Timestamp,
}

impl Ec2SecurityCredentialsMetadataResponse {
    /// Deserialize a JSON response received from the EC2 metadata service.
    ///
    /// Parses the credentials from a response received from
    /// `http://169.254.169.254/latest/meta-data/iam/security-credentials/{name-of-IAM-role}`.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is invalid.
    pub fn deserialize(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Get the key of this `Ec2SecurityCredentialsMetadataResponse`
    #[inline]
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the secret of this `Ec2SecurityCredentialsMetadataResponse`
    #[inline]
    #[must_use]
    pub fn secret(&self) -> &str {
        &self.secret
    }

    /// Get the token of this `Ec2SecurityCredentialsMetadataResponse`
    #[inline]
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Get the expiration of the credentials of this `Ec2SecurityCredentialsMetadataResponse`
    #[inline]
    #[must_use]
    pub const fn expiration(&self) -> Timestamp {
        self.expiration
    }

    /// Convert this `Ec2SecurityCredentialsMetadataResponse` into [`Credentials`]
    #[inline]
    #[must_use]
    pub fn into_credentials(mut self) -> Credentials {
        let key = mem::take(&mut self.key);
        let secret = mem::take(&mut self.secret);
        let token = mem::take(&mut self.token);
        Credentials::new_with_token(key, secret, token)
    }

    /// Update a [`RotatingCredentials`] with the credentials of this `Ec2SecurityCredentialsMetadataResponse`
    #[inline]
    pub fn rotate_credentials(mut self, rotating: &RotatingCredentials) {
        let key = mem::take(&mut self.key);
        let secret = mem::take(&mut self.secret);
        let token = mem::take(&mut self.token);
        rotating.update(key, secret, Some(token));
    }
}

impl Debug for Ec2SecurityCredentialsMetadataResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ec2SecurityCredentialsMetadataResponse")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl Drop for Ec2SecurityCredentialsMetadataResponse {
    fn drop(&mut self) {
        self.secret.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const JSON: &str = r#"{
    "Code" : "Success",
    "LastUpdated" : "2020-12-28T16:47:50Z",
    "Type" : "AWS-HMAC",
    "AccessKeyId" : "some_access_key",
    "SecretAccessKey" : "some_secret_key",
    "Token" : "some_token",
    "Expiration" : "2020-12-28T23:10:09Z"
}"#;

    #[test]
    fn deserialize() {
        let deserialized = Ec2SecurityCredentialsMetadataResponse::deserialize(JSON).unwrap();
        assert_eq!(deserialized.key(), "some_access_key");
        assert_eq!(deserialized.secret(), "some_secret_key");
        assert_eq!(deserialized.token(), "some_token");
        //                                                                  2020-12-28T23:10:09Z
        assert_eq!(
            deserialized
                .expiration()
                .duration_since(Timestamp::UNIX_EPOCH)
                .as_secs(),
            1_609_197_009
        );

        let debug_output = format!("{deserialized:?}");
        assert_eq!(
            debug_output,
            "Ec2SecurityCredentialsMetadataResponse { key: \"some_access_key\", .. }"
        );
    }

    #[test]
    fn into_credentials() {
        let deserialized = Ec2SecurityCredentialsMetadataResponse::deserialize(JSON).unwrap();

        let credentials = deserialized.into_credentials();
        assert_eq!(credentials.key(), "some_access_key");
        assert_eq!(credentials.secret(), "some_secret_key");
        assert_eq!(credentials.token(), Some("some_token"));
    }

    #[test]
    fn rotate_credentials() {
        let rotating = RotatingCredentials::new("abcd".into(), "1234".into(), None);
        let deserialized = Ec2SecurityCredentialsMetadataResponse::deserialize(JSON).unwrap();

        deserialized.rotate_credentials(&rotating);

        let current = rotating.get();
        assert_eq!(current.key(), "some_access_key");
        assert_eq!(current.secret(), "some_secret_key");
        assert_eq!(current.token(), Some("some_token"));
    }
}
