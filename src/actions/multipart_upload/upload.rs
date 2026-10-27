use std::time::Duration;

use jiff::Timestamp;
use url::Url;

use crate::actions::Method;
use crate::actions::S3Action;
use crate::signing::sign;
use crate::sorting_iter::SortingIterator;
use crate::{Bucket, Credentials, Map};

/// Upload a part to a previously created multipart upload.
///
/// Every part must be between 5 MiB and 5 GiB in size, except for the last
/// part, which can be smaller.
///
/// The part must be uploaded via a PUT request, on success the server will
/// return an `ETag` header which must be given to
/// [`CompleteMultipartUpload`][crate::actions::CompleteMultipartUpload] in order to
/// complete the upload.
///
/// A maximum of 10,000 parts can be uploaded to a single multipart upload.
///
/// ## Choosing the size of parts
///
/// Since there can only be 10,000 parts, they must be at least as large as
/// the object divided by 10,000: for example, 5 MiB parts can upload objects
/// of up to about 48.8 GiB. Within these limits, smaller parts:
///
/// * can be uploaded concurrently over more connections
/// * lose less progress when uploading one of them fails and has to be
///   retried
/// * work over slow connections: some S3 compatible services abort requests
///   which upload slower than a minimum speed, so parts of 5 to 10 MiB might
///   be the only way to upload an object at all
///
/// while larger parts take fewer requests, each of them signed, sent and,
/// on AWS, paid for separately. The AWS CLI uses 8 MiB parts, making them
/// larger only for objects which would need more than 10,000 of them.
///
/// The uploaded part will consume storage on S3 until the multipart upload
/// is completed or aborted.
///
/// Find out more about `UploadPart` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPart.html
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct UploadPart<'a> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,
    object: &'a str,

    part_number: u16,
    upload_id: &'a str,

    query: Map<'a>,
    headers: Map<'a>,
}

impl<'a> UploadPart<'a> {
    /// Construct a new `UploadPart` action
    ///
    /// Without `credentials` the request isn't signed.
    #[inline]
    #[must_use]
    pub const fn new(
        bucket: &'a Bucket,
        credentials: Option<&'a Credentials>,
        object: &'a str,
        part_number: u16,
        upload_id: &'a str,
    ) -> Self {
        Self {
            bucket,
            credentials,
            object,

            part_number,
            upload_id,

            query: Map::new(),
            headers: Map::new(),
        }
    }
}

impl<'a> S3Action<'a> for UploadPart<'a> {
    const METHOD: Method = Method::Put;

    fn query_mut(&mut self) -> &mut Map<'a> {
        &mut self.query
    }

    fn headers_mut(&mut self) -> &mut Map<'a> {
        &mut self.headers
    }

    fn sign_with_time(&self, expires_in: Duration, time: &Timestamp) -> Url {
        let url = self.bucket.object_url(self.object);

        let part_number = self.part_number.to_string();
        let query = [
            ("partNumber", part_number.as_str()),
            ("uploadId", self.upload_id),
        ];
        let query = SortingIterator::new(query.into_iter(), self.query.iter());

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

        let action = UploadPart::new(&bucket, Some(&credentials), "test.txt", 1, "abcd");

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&partNumber=1&uploadId=abcd&X-Amz-Signature=d2ed12e1e116c88a79cd6d1726f5fe75c99db8a0292ba000f97ecc309a9303f8";

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

        let mut action = UploadPart::new(&bucket, None, "test.txt", 1, "abcd");
        action.query_mut().insert("x-id", "UploadPart");

        let url = action.sign(expires_in);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?partNumber=1&uploadId=abcd&x-id=UploadPart";

        assert_eq!(expected, url.as_str());
    }
}
