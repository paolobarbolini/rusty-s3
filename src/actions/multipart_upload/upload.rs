use std::iter;
use std::time::Duration;

use time::OffsetDateTime;
use url::Url;

use crate::actions::Method;
use crate::actions::S3Action;
use crate::signing::sign;
use crate::{Bucket, Credentials};

/// Upload a part to a previously created multipart upload.
///
/// Every part must be between 5 MB and 5 GB in size, except for the last part.
///
/// The part must be uploaded via a PUT request, on success the server will
/// return an `ETag` header which must be given to
/// [`CompleteMultipartUpload`][crate::actions::CompleteMultipartUpload] in order to
/// complete the upload.
///
/// A maximum of 10,000 parts can be uploaded to a single multipart upload.
///
/// The uploaded part will consume storage on S3 until the multipart upload
/// is completed or aborted.
///
/// Find out more about `UploadPart` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPart.html
#[derive(Debug, Clone)]
pub struct UploadPart<'a> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,
    object: &'a str,

    part_number: u16,
    upload_id: &'a str,
}

impl<'a> UploadPart<'a> {
    #[inline]
    pub fn new(
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
        }
    }

    fn sign_with_time(&self, expires_in: Duration, time: &OffsetDateTime) -> Url {
        let url = self.bucket.object_url(self.object).unwrap();

        let part_number = self.part_number.to_string();
        let query = [
            ("partNumber", part_number.as_str()),
            ("uploadId", self.upload_id),
        ];

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
                query.iter().copied(),
                iter::empty(),
            ),
            None => crate::signing::util::add_query_params(url, query.iter().copied()),
        }
    }
}

impl<'a> S3Action for UploadPart<'a> {
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

        let action = UploadPart::new(&bucket, Some(&credentials), "test.txt", 1, "abcd");

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&partNumber=1&uploadId=abcd&X-Amz-Signature=d2ed12e1e116c88a79cd6d1726f5fe75c99db8a0292ba000f97ecc309a9303f8";

        assert_eq!(expected, url.as_str());
    }

    #[test]
    fn anonymous_custom_query() {
        let expires_in = Duration::from_secs(86400);

        let endpoint = "https://s3.amazonaws.com".parse().unwrap();
        let bucket =
            Bucket::new(endpoint, false, "examplebucket".into(), "us-east-1".into()).unwrap();

        let action = UploadPart::new(&bucket, None, "test.txt", 1, "abcd");
        let url = action.sign(expires_in);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?partNumber=1&uploadId=abcd";

        assert_eq!(expected, url.as_str());
    }
}
