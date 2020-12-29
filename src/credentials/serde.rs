use std::fmt::{self, Debug, Formatter};

use serde::{Deserialize, Deserializer};
use time::PrimitiveDateTime;

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
    #[serde(rename = "Expiration", deserialize_with = "expiration_deserializer")]
    expiration: PrimitiveDateTime,
}

fn expiration_deserializer<'de, D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    PrimitiveDateTime::parse(s, "%Y-%m-%dT%H:%M:%SZ").map_err(serde::de::Error::custom)
}

impl Ec2SecurityCredentialsMetadataResponse {
    /// Deserialize a JSON response received from the EC2 metadata service.
    ///
    /// Parses the credentials from a response received from
    /// `http://169.254.169.254/latest/meta-data/iam/security-credentials/{name-of-IAM-role}`.
    pub fn deserialize(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Get the key of this `Ec2SecurityCredentialsMetadataResponse`
    #[inline]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the secret of this `Ec2SecurityCredentialsMetadataResponse`
    #[inline]
    pub fn secret(&self) -> &str {
        &self.secret
    }

    /// Get the token of this `Ec2SecurityCredentialsMetadataResponse`
    #[inline]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Get the expiration of the credentials of this `Ec2SecurityCredentialsMetadataResponse`
    #[inline]
    pub fn expiration(&self) -> PrimitiveDateTime {
        self.expiration
    }

    /// Convert this `Ec2SecurityCredentialsMetadataResponse` into [`Credentials`]
    #[inline]
    pub fn into_credentials(self) -> Credentials {
        Credentials::new_(self.key, self.secret, Some(self.token))
    }

    /// Update a [`RotatingCredentials`] with the credentials of this `Ec2SecurityCredentialsMetadataResponse`
    #[inline]
    pub fn rotate_credentials(self, rotating: &RotatingCredentials) {
        rotating.update(self.key, self.secret, Some(self.token));
    }
}

impl Debug for Ec2SecurityCredentialsMetadataResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ec2SecurityCredentialsMetadataResponse")
            .field("key", &self.key)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn deserialize() {
        let json = r#"{
    "Code" : "Success",
    "LastUpdated" : "2020-12-28T16:47:50Z",
    "Type" : "AWS-HMAC",
    "AccessKeyId" : "some_access_key",
    "SecretAccessKey" : "some_secret_key",
    "Token" : "some_token",
    "Expiration" : "2020-12-28T23:10:09Z"
}"#;

        let deserialized = Ec2SecurityCredentialsMetadataResponse::deserialize(json).unwrap();
        assert_eq!(deserialized.key(), "some_access_key");
        assert_eq!(deserialized.secret(), "some_secret_key");
        assert_eq!(deserialized.token(), "some_token");
        assert_eq!(
            deserialized.expiration().format("%Y-%m-%dT%H:%M:%SZ"),
            "2020-12-28T23:10:09Z"
        );

        let debug_output = format!("{:?}", deserialized);
        assert_eq!(
            debug_output,
            "Ec2SecurityCredentialsMetadataResponse { key: \"some_access_key\" }"
        );
    }
}
