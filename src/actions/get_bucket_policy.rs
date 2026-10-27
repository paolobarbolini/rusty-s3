use std::iter;
use std::time::Duration;

use jiff::Timestamp;
use serde::Deserialize;
use url::Url;

use super::S3Action;
use crate::actions::Method;
use crate::signing::sign;
use crate::sorting_iter::SortingIterator;
use crate::{Bucket, Credentials, Map};

const POLICY_PARAM: &str = "policy";

/// Retrieve a bucket's policy from S3.
///
/// Find out more about `GetBucketPolicy` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketPolicy.html
#[derive(Debug, Clone)]
pub struct GetBucketPolicy<'a> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,

    query: Map<'a>,
    headers: Map<'a>,
}

/// The policy of a bucket, as returned by [`GetBucketPolicy`]
///
/// Only the `Version` and `Id` of the policy are parsed.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct GetBucketPolicyResponse {
    /// The version of the policy language, empty if the policy doesn't have one
    #[serde(rename = "Version", default)]
    pub version: String,
    /// The ID of the policy, if it has one
    #[serde(rename = "Id")]
    pub id: Option<String>,
}

impl<'a> GetBucketPolicy<'a> {
    /// Construct a new `GetBucketPolicy` action
    ///
    /// Without `credentials` the request isn't signed.
    #[inline]
    #[must_use]
    pub const fn new(bucket: &'a Bucket, credentials: Option<&'a Credentials>) -> Self {
        Self {
            bucket,
            credentials,

            query: Map::new(),
            headers: Map::new(),
        }
    }

    /// Parse the response from S3.
    ///
    /// # Errors
    ///
    /// If the response cannot be parsed.
    pub fn parse_response(s: &str) -> Result<GetBucketPolicyResponse, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl<'a> S3Action<'a> for GetBucketPolicy<'a> {
    const METHOD: Method = Method::Get;

    fn query_mut(&mut self) -> &mut Map<'a> {
        &mut self.query
    }

    fn headers_mut(&mut self) -> &mut Map<'a> {
        &mut self.headers
    }

    fn sign_with_time(&self, expires_in: Duration, time: &Timestamp) -> Url {
        let url = self.bucket.base_url().clone();
        let query = SortingIterator::new(iter::once((POLICY_PARAM, "")), self.query.iter());

        match self.credentials {
            Some(credentials) => sign(
                time,
                Self::METHOD,
                url,
                credentials.key(),
                credentials.secret(),
                credentials.token(),
                self.bucket.region(),
                expires_in.as_secs(),
                query,
                self.headers.iter(),
            ),
            None => crate::signing::util::add_query_params(url, query),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{Bucket, Credentials, UrlStyle};

    #[test]
    fn aws_example_signed() {
        // Fri, 24 May 2013 00:00:00 GMT
        let date = Timestamp::from_second(1_369_353_600).unwrap();
        let expires_in = Duration::from_secs(86400);

        let endpoint = "https://s3.amazonaws.com".parse().unwrap();
        let bucket = Bucket::new(
            endpoint,
            UrlStyle::VirtualHost,
            "examplebucket",
            "us-east-1",
        )
        .unwrap();
        let credentials = Credentials::new(
            "AKIAIOSFODNN7EXAMPLE",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        );

        let action = GetBucketPolicy::new(&bucket, Some(&credentials));

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&policy=&X-Amz-Signature=e26de56ab8c3baaf62acf68f921e280f4674f741813006644bbd5fb6d2aaa2eb";

        assert_eq!(expected, url.as_str());
    }

    #[test]
    fn anonymous_custom_query() {
        let expires_in = Duration::from_secs(86400);

        let endpoint = "https://s3.amazonaws.com".parse().unwrap();
        let bucket = Bucket::new(
            endpoint,
            UrlStyle::VirtualHost,
            "examplebucket",
            "us-east-1",
        )
        .unwrap();

        let action = GetBucketPolicy::new(&bucket, None);
        let url = action.sign(expires_in);
        let expected = "https://examplebucket.s3.amazonaws.com/?policy=";
        assert_eq!(expected, url.as_str());

        let mut action = GetBucketPolicy::new(&bucket, None);
        action.query_mut().insert("x-id", "GetBucketPolicy");
        let url = action.sign(expires_in);
        let expected = "https://examplebucket.s3.amazonaws.com/?policy=&x-id=GetBucketPolicy";
        assert_eq!(expected, url.as_str());
    }

    #[test]
    fn parse_response_no_version() -> Result<(), serde_json::Error> {
        // `Version` is optional in the policy grammar
        let content = r#"{
"Id":"aaaa-bbbb-cccc-dddd",
"Statement" : [
    {
        "Effect":"Allow",
        "Principal":"*",
        "Action":"s3:GetObject",
        "Resource":"arn:aws:s3:::bucket/*"
    }
]
}
"#;
        assert_eq!(
            GetBucketPolicy::parse_response(content)?,
            GetBucketPolicyResponse {
                version: String::new(),
                id: Some("aaaa-bbbb-cccc-dddd".to_string()),
            }
        );
        Ok(())
    }

    #[test]
    fn aws_example() -> Result<(), serde_json::Error> {
        assert_eq!(
            GetBucketPolicy::parse_response(r#"{"Version":"1"}"#)?,
            GetBucketPolicyResponse {
                version: "1".to_string(),
                id: None
            }
        );

        let content = r#"{
"Version":"2008-10-17",
"Id":"aaaa-bbbb-cccc-dddd",
"Statement" : [
    {
        "Effect":"Deny",
        "Sid":"1",
        "Principal" : {
            "AWS":["111122223333","444455556666"]
        },
        "Action":["s3:*"],
        "Resource":"arn:aws:s3:::bucket/*"
    }
]
}
"#;
        assert_eq!(
            GetBucketPolicy::parse_response(content)?,
            GetBucketPolicyResponse {
                version: "2008-10-17".to_string(),
                id: Some("aaaa-bbbb-cccc-dddd".to_string()),
            }
        );
        Ok(())
    }
}
