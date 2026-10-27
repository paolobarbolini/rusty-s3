use std::time::Duration;

#[cfg(feature = "xml")]
use instant_xml::ToXml;
use jiff::Timestamp;
use url::Url;

use crate::actions::Method;
#[cfg(feature = "xml")]
use crate::actions::S3_XML_NS;
use crate::actions::S3Action;
use crate::signing::sign;
use crate::{Bucket, Credentials, Map};

/// Create a new bucket.
///
/// Outside of `us-east-1`, the region of the bucket must also be sent as
/// its location constraint in the body of the request, which
/// `CreateBucket::body` generates when the `xml` feature is enabled.
///
/// Find out more about `CreateBucket` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateBucket.html
#[derive(Debug, Clone)]
pub struct CreateBucket<'a> {
    bucket: &'a Bucket,
    credentials: &'a Credentials,

    query: Map<'a>,
    headers: Map<'a>,
}

impl<'a> CreateBucket<'a> {
    /// Construct a new `CreateBucket` action
    #[must_use]
    pub const fn new(bucket: &'a Bucket, credentials: &'a Credentials) -> Self {
        Self {
            bucket,
            credentials,

            query: Map::new(),
            headers: Map::new(),
        }
    }

    /// Generate the XML body for the request.
    ///
    /// It sets the region of the bucket as its location constraint, which is
    /// required in every region except `us-east-1`, where it must be left
    /// out, so the body is empty.
    ///
    /// # Panics
    ///
    /// Panics if XML serialization fails, which shouldn't happen in practice.
    #[cfg(feature = "xml")]
    #[must_use]
    pub fn body(&self) -> String {
        #[derive(ToXml)]
        #[xml(rename = "CreateBucketConfiguration", ns(S3_XML_NS))]
        struct CreateBucketConfiguration<'a> {
            #[xml(rename = "LocationConstraint")]
            location_constraint: &'a str,
        }

        let region = self.bucket.region();
        if region == "us-east-1" {
            return String::new();
        }

        let req = CreateBucketConfiguration {
            location_constraint: region,
        };
        instant_xml::to_string(&req).unwrap()
    }
}

impl<'a> S3Action<'a> for CreateBucket<'a> {
    const METHOD: Method = Method::Put;

    fn query_mut(&mut self) -> &mut Map<'a> {
        &mut self.query
    }

    fn headers_mut(&mut self) -> &mut Map<'a> {
        &mut self.headers
    }

    fn sign_with_time(&self, expires_in: Duration, time: &Timestamp) -> Url {
        let url = self.bucket.base_url().clone();

        sign(
            time,
            Self::METHOD,
            url,
            self.credentials.key(),
            self.credentials.secret(),
            self.credentials.token(),
            self.bucket.region(),
            expires_in.as_secs(),
            self.query.iter(),
            self.headers.iter(),
        )
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{Bucket, Credentials, UrlStyle};

    #[test]
    fn aws_example() {
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

        let action = CreateBucket::new(&bucket, &credentials);

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&X-Amz-Signature=fb5c8ab11e9fd9d3c54ea0293e1df0820feef6c1f2de12e5fe00636e3f0cf9d2";

        assert_eq!(expected, url.as_str());

        #[cfg(feature = "xml")]
        assert_eq!(action.body(), "");
    }

    #[test]
    #[cfg(feature = "xml")]
    fn body_location_constraint() {
        let endpoint = "https://s3.eu-west-1.amazonaws.com".parse().unwrap();
        let bucket = Bucket::new(
            endpoint,
            UrlStyle::VirtualHost,
            "examplebucket",
            "eu-west-1",
        )
        .unwrap();
        let credentials = Credentials::new(
            "AKIAIOSFODNN7EXAMPLE",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        );

        let action = CreateBucket::new(&bucket, &credentials);
        let expected = "<CreateBucketConfiguration xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\"><LocationConstraint>eu-west-1</LocationConstraint></CreateBucketConfiguration>";

        assert_eq!(action.body(), expected);
    }
}
