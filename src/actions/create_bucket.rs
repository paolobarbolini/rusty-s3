use std::iter;
use std::time::Duration;

use time::OffsetDateTime;
use url::Url;

use crate::actions::Method;
use crate::actions::S3Action;
use crate::signing::sign;
use crate::{Bucket, Credentials, Map};

/// Create a new bucket.
///
/// Find out more about `CreateBucket` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateBucket.html
#[derive(Debug, Clone)]
pub struct CreateBucket<'a> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,

    query: Map<'a>,
}

impl<'a> CreateBucket<'a> {
    // TODO: don't take an Option for Credentials, since CreateBucket requests must be authenticated.
    pub fn new(bucket: &'a Bucket, credentials: Option<&'a Credentials>) -> Self {
        Self {
            bucket,
            credentials,

            query: Map::new(),
        }
    }

    pub fn query_mut(&mut self) -> &mut Map<'a> {
        &mut self.query
    }

    fn sign_with_time(&self, expires_in: Duration, time: &OffsetDateTime) -> Url {
        let url = self.bucket.base_url().clone();

        match self.credentials {
            Some(credentials) => sign(
                time,
                Method::Put,
                url,
                credentials.key(),
                credentials.secret(),
                credentials.token(),
                self.bucket.region(),
                expires_in.as_secs(),
                self.query.iter(),
                iter::empty(),
            ),
            None => crate::signing::util::add_query_params(url, self.query.iter()),
        }
    }
}

impl<'a> S3Action for CreateBucket<'a> {
    const METHOD: Method = Method::Put;

    fn sign(&self, expires_in: Duration) -> Url {
        let now = OffsetDateTime::now_utc();
        self.sign_with_time(expires_in, &now)
    }
}

#[cfg(test)]
mod tests {
    use time::PrimitiveDateTime;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{Bucket, Credentials};

    #[test]
    fn aws_example() {
        let date = PrimitiveDateTime::parse(
            "Fri, 24 May 2013 00:00:00 GMT",
            "%a, %d %b %Y %-H:%M:%S GMT",
        )
        .unwrap()
        .assume_utc();
        let expires_in = Duration::from_secs(86400);

        let endpoint = "https://s3.amazonaws.com".parse().unwrap();
        let bucket =
            Bucket::new(endpoint, false, "examplebucket".into(), "us-east-1".into()).unwrap();
        let credentials = Credentials::new(
            "AKIAIOSFODNN7EXAMPLE".into(),
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".into(),
        );

        let action = CreateBucket::new(&bucket, Some(&credentials));

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&X-Amz-Signature=fb5c8ab11e9fd9d3c54ea0293e1df0820feef6c1f2de12e5fe00636e3f0cf9d2";

        assert_eq!(expected, url.as_str());
    }

    #[test]
    fn anonymous_custom_query() {
        let expires_in = Duration::from_secs(86400);

        let endpoint = "https://s3.amazonaws.com".parse().unwrap();
        let bucket =
            Bucket::new(endpoint, false, "examplebucket".into(), "us-east-1".into()).unwrap();

        let mut action = CreateBucket::new(&bucket, None);
        action.query_mut().insert("x-amz-grant-read", "things");

        let url = action.sign(expires_in);
        let expected = "https://examplebucket.s3.amazonaws.com/?x-amz-grant-read=things";

        assert_eq!(expected, url.as_str());
    }
}
