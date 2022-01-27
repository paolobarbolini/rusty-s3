use std::borrow::{Borrow, Cow};
use std::iter;
use std::time::Duration;

use time::OffsetDateTime;
use url::Url;

use super::S3Action;
use crate::actions::Method;
use crate::signing::sign;
use crate::sorting_iter::SortingIterator;
use crate::{Bucket, Credentials, Map};

/// Create a copy of an object that is already stored in S3, using a `PUT` request.
///
/// Note that:
/// * only objects up to 5 GB can be copied using this method
/// * even if the server returns a 200 response the copy might have failed
///
/// Find out more about CopyObject from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html
#[derive(Debug, Clone)]
pub struct CopyObject<'a> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,
    src_object: &'a str,
    dst_object: &'a str,
    prepend_bucket: bool,

    query: Map<'a>,
    headers: Map<'a>,
}

impl<'a> CopyObject<'a> {
    #[inline]
    pub fn new(
        bucket: &'a Bucket,
        credentials: Option<&'a Credentials>,
        src_object: &'a str,
        dst_object: &'a str,
        prepend_bucket: bool,
    ) -> Self {
        Self {
            bucket,
            credentials,
            src_object,
            dst_object,
            prepend_bucket,

            query: Map::new(),
            headers: Map::new(),
        }
    }
}

impl<'a> S3Action<'a> for CopyObject<'a> {
    const METHOD: Method = Method::Put;

    fn sign(&self, expires_in: Duration) -> Url {
        let now = OffsetDateTime::now_utc();
        self.sign_with_time(expires_in, &now)
    }

    fn query_mut(&mut self) -> &mut Map<'a> {
        &mut self.query
    }

    fn headers_mut(&mut self) -> &mut Map<'a> {
        &mut self.headers
    }

    fn sign_with_time(&self, expires_in: Duration, time: &OffsetDateTime) -> Url {
        let url = self.bucket.object_url(self.dst_object).unwrap();
        let copy_source = if self.prepend_bucket {
            Cow::from(format!("{}/{}", self.bucket.name(), self.src_object))
        } else {
            Cow::from(self.src_object)
        };
        let query = SortingIterator::new(
            iter::once(("x-amz-copy-source", copy_source.borrow())),
            self.query.iter(),
        );

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
                query,
                self.headers.iter(),
            ),
            None => crate::signing::util::add_query_params(url, query),
        }
    }
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{Bucket, Credentials, UrlStyle};

    #[test]
    fn aws_example() {
        // Fri, 24 May 2013 00:00:00 GMT
        let date = OffsetDateTime::from_unix_timestamp(1369353600).unwrap();
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

        let action = CopyObject::new(
            &bucket,
            Some(&credentials),
            "test.txt",
            "test_copy.txt",
            true,
        );

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/test_copy.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&x-amz-copy-source=examplebucket%2Ftest.txt&X-Amz-Signature=760326dbb90c424f6b5dcfa5f8473754f44cb4c05c173416feb1b9306dc64d35";

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

        let action = CopyObject::new(&bucket, None, "test.txt", "test_copy.txt", true);
        let url = action.sign(expires_in);
        let expected = "https://examplebucket.s3.amazonaws.com/test_copy.txt?x-amz-copy-source=examplebucket%2Ftest.txt";

        assert_eq!(expected, url.as_str());
    }
}
