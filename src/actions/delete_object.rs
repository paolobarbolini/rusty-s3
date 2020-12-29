use std::iter;
use std::time::Duration;

use time::OffsetDateTime;
use url::Url;

use super::S3Action;
use crate::actions::Method;
use crate::signing::sign;
use crate::{Bucket, Credentials};

/// Delete an object from S3, using a `DELETE` request.
///
/// Find out more about `DeleteObject` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObject.html
#[derive(Debug, Clone)]
pub struct DeleteObject<'a> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,
    object: &'a str,
}

impl<'a> DeleteObject<'a> {
    #[inline]
    pub fn new(bucket: &'a Bucket, credentials: Option<&'a Credentials>, object: &'a str) -> Self {
        Self {
            bucket,
            credentials,
            object,
        }
    }

    fn sign_with_time(&self, expires_in: Duration, time: &OffsetDateTime) -> Url {
        let url = self.bucket.object_url(self.object).unwrap();

        match self.credentials {
            Some(credentials) => sign(
                time,
                Method::Delete,
                url,
                credentials.key(),
                credentials.secret(),
                credentials.token(),
                self.bucket.region(),
                expires_in.as_secs(),
                iter::empty(),
                iter::empty(),
            ),
            None => url,
        }
    }
}

impl<'a> S3Action for DeleteObject<'a> {
    const METHOD: Method = Method::Delete;

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

        let action = DeleteObject::new(&bucket, Some(&credentials), "test.txt");

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&X-Amz-Signature=fb580faa6736a3af12ad5f9c3f1eea783af940a06f6a3de9dadb5679ca25cbfe";

        assert_eq!(expected, url.as_str());
    }

    #[test]
    fn anonymous_custom_query() {
        let expires_in = Duration::from_secs(86400);

        let endpoint = "https://s3.amazonaws.com".parse().unwrap();
        let bucket =
            Bucket::new(endpoint, false, "examplebucket".into(), "us-east-1".into()).unwrap();

        let action = DeleteObject::new(&bucket, None, "test.txt");
        let url = action.sign(expires_in);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt";

        assert_eq!(expected, url.as_str());
    }
}
