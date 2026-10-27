use std::iter;
use std::time::Duration;

use jiff::Timestamp;
use url::Url;

use crate::actions::Method;
use crate::actions::S3Action;
use crate::signing::sign;
use crate::sorting_iter::SortingIterator;
use crate::{Bucket, Credentials, Map};

/// Abort multipart upload.
///
/// This frees the storage used by the parts which were already uploaded.
/// A part which is still being uploaded when the abort is processed might
/// still be stored, and keep using storage, so wait for every
/// [`UploadPart`][crate::actions::UploadPart] request to complete before
/// aborting. AWS also recommends checking with `ListParts` that no parts are
/// left, aborting again if some are.
///
/// Uploads which fail aren't always aborted, like when the process uploading
/// them crashes, so the bucket should also have a lifecycle rule which
/// aborts the uploads left incomplete for a number of days, as a fail-safe:
/// see [Configuring a bucket lifecycle configuration to delete incomplete multipart uploads][lifecycle].
///
/// Find out more about `AbortMultipartUpload` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_AbortMultipartUpload.html
/// [lifecycle]: https://docs.aws.amazon.com/AmazonS3/latest/userguide/mpu-abort-incomplete-mpu-lifecycle-config.html
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct AbortMultipartUpload<'a> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,
    object: &'a str,

    upload_id: &'a str,

    query: Map<'a>,
    headers: Map<'a>,
}

impl<'a> AbortMultipartUpload<'a> {
    #[inline]
    #[must_use]
    pub const fn new(
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

            query: Map::new(),
            headers: Map::new(),
        }
    }
}

impl<'a> S3Action<'a> for AbortMultipartUpload<'a> {
    const METHOD: Method = Method::Delete;

    fn query_mut(&mut self) -> &mut Map<'a> {
        &mut self.query
    }

    fn headers_mut(&mut self) -> &mut Map<'a> {
        &mut self.headers
    }

    fn sign_with_time(&self, expires_in: Duration, time: &Timestamp) -> Url {
        let url = self.bucket.object_url(self.object);
        let query =
            SortingIterator::new(iter::once(("uploadId", self.upload_id)), self.query.iter());

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

        let action = AbortMultipartUpload::new(&bucket, Some(&credentials), "test.txt", "abcd");

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&uploadId=abcd&X-Amz-Signature=7670bc768a7cdb5c276a9dddadeefdffb52061f94db6c14b4a9284fdc195bb59";

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

        let mut action = AbortMultipartUpload::new(&bucket, None, "test.txt", "abcd");
        action.query_mut().insert("x-id", "AbortMultipartUpload");

        let url = action.sign(expires_in);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?uploadId=abcd&x-id=AbortMultipartUpload";

        assert_eq!(expected, url.as_str());
    }
}
