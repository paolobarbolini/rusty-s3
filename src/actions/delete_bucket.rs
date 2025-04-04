use std::time::Duration;

use jiff::Timestamp;
use url::Url;

use crate::actions::Method;
use crate::actions::S3Action;
use crate::signing::sign;
use crate::{Bucket, Credentials, Map};

/// Delete a bucket.
///
/// The bucket must be empty before it can be deleted.
///
/// Find out more about `DeleteBucket` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucket.html
#[derive(Debug, Clone)]
pub struct DeleteBucket<'a> {
    bucket: &'a Bucket,
    credentials: &'a Credentials,

    query: Map<'a>,
    headers: Map<'a>,
}

impl<'a> DeleteBucket<'a> {
    #[must_use]
    pub const fn new(bucket: &'a Bucket, credentials: &'a Credentials) -> Self {
        Self {
            bucket,
            credentials,

            query: Map::new(),
            headers: Map::new(),
        }
    }
}

impl<'a> S3Action<'a> for DeleteBucket<'a> {
    const METHOD: Method = Method::Delete;

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
        let date = Timestamp::from_second(1369353600).unwrap();
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

        let action = DeleteBucket::new(&bucket, &credentials);

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&X-Amz-Signature=875ca449635876849f9cf1622dc709f1978d82e7f6e067b173e6212e3850a1e9";

        assert_eq!(expected, url.as_str());
    }
}
