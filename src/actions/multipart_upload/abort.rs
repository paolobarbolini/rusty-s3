use std::iter;
use std::time::Duration;

use time::OffsetDateTime;
use url::Url;

use crate::actions::Method;
use crate::actions::S3Action;
use crate::signing::sign;
use crate::{Bucket, Credentials};

/// Abort multipart upload.
///
/// This also cleans up any previously uploaded part.
///
/// Find out more about `AbortMultipartUpload` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_AbortMultipartUpload.html
#[derive(Debug, Clone)]
pub struct AbortMultipartUpload<'a> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,
    object: &'a str,

    upload_id: &'a str,
}

impl<'a> AbortMultipartUpload<'a> {
    #[inline]
    pub fn new(
        bucket: &'a Bucket,
        credentials: Option<&'a Credentials>,
        object: &'a str,
        upload_id: &'a str,
    ) -> Self {
        Self {
            bucket,
            credentials,
            object,

            upload_id,
        }
    }

    fn sign_with_time(&self, expires_in: Duration, time: &OffsetDateTime) -> Url {
        let url = self.bucket.object_url(self.object).unwrap();
        let query = iter::once(("uploadId", self.upload_id));

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
                query,
                iter::empty(),
            ),
            None => crate::signing::util::add_query_params(url, query),
        }
    }
}

impl<'a> S3Action for AbortMultipartUpload<'a> {
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

        let action = AbortMultipartUpload::new(&bucket, Some(&credentials), "test.txt", "abcd");

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&uploadId=abcd&X-Amz-Signature=7670bc768a7cdb5c276a9dddadeefdffb52061f94db6c14b4a9284fdc195bb59";

        assert_eq!(expected, url.as_str());
    }

    #[test]
    fn anonymous_custom_query() {
        let expires_in = Duration::from_secs(86400);

        let endpoint = "https://s3.amazonaws.com".parse().unwrap();
        let bucket =
            Bucket::new(endpoint, false, "examplebucket".into(), "us-east-1".into()).unwrap();

        let action = AbortMultipartUpload::new(&bucket, None, "test.txt", "abcd");
        let url = action.sign(expires_in);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?uploadId=abcd";

        assert_eq!(expected, url.as_str());
    }
}
