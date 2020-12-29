use std::iter;
use std::time::Duration;

use serde::Deserialize;
use time::OffsetDateTime;
use url::Url;

use crate::actions::Method;
use crate::actions::S3Action;
use crate::signing::sign;
use crate::{Bucket, Credentials};

/// Create a multipart upload.
///
/// A few advantages of multipart uploads are:
///
/// * being able to be resume without having to start back from the beginning
/// * parallelize the uploads across multiple threads
///
/// Find out more about `CreateMultipartUpload` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html
#[derive(Debug, Clone)]
pub struct CreateMultipartUpload<'a> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,
    object: &'a str,
}

#[derive(Debug, Clone)]
pub struct CreateMultipartUploadResponse(InnerCreateMultipartUploadResponse);

#[derive(Debug, Clone, Deserialize)]
struct InnerCreateMultipartUploadResponse {
    #[serde(rename = "UploadId")]
    upload_id: String,
}

impl<'a> CreateMultipartUpload<'a> {
    #[inline]
    pub fn new(bucket: &'a Bucket, credentials: Option<&'a Credentials>, object: &'a str) -> Self {
        Self {
            bucket,
            credentials,
            object,
        }
    }

    pub fn parse_response(s: &str) -> Result<CreateMultipartUploadResponse, quick_xml::DeError> {
        let parsed = quick_xml::de::from_str(s)?;
        Ok(CreateMultipartUploadResponse(parsed))
    }

    fn sign_with_time(&self, expires_in: Duration, time: &OffsetDateTime) -> Url {
        let url = self.bucket.object_url(self.object).unwrap();
        let query = iter::once(("uploads", "1"));

        match self.credentials {
            Some(credentials) => sign(
                time,
                Method::Post,
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

impl CreateMultipartUploadResponse {
    pub fn upload_id(&self) -> &str {
        &self.0.upload_id
    }
}

impl<'a> S3Action for CreateMultipartUpload<'a> {
    const METHOD: Method = Method::Post;

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

        let action = CreateMultipartUpload::new(&bucket, Some(&credentials), "test.txt");

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&uploads=1&X-Amz-Signature=a6289f9e5ff2a914c6e324403bcd00b1d258c568487faa50d317ef0910c25c0a";

        assert_eq!(expected, url.as_str());
    }

    #[test]
    fn anonymous_custom_query() {
        let expires_in = Duration::from_secs(86400);

        let endpoint = "https://s3.amazonaws.com".parse().unwrap();
        let bucket =
            Bucket::new(endpoint, false, "examplebucket".into(), "us-east-1".into()).unwrap();

        let action = CreateMultipartUpload::new(&bucket, None, "test.txt");
        let url = action.sign(expires_in);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?uploads=1";

        assert_eq!(expected, url.as_str());
    }
}
