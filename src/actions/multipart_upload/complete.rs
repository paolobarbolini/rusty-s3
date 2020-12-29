use std::iter;
use std::time::Duration;

use serde::Serialize;
use time::OffsetDateTime;
use url::Url;

use crate::actions::Method;
use crate::actions::S3Action;
use crate::signing::sign;
use crate::{Bucket, Credentials};

/// Complete a multipart upload.
///
/// Find out more about `CompleteMultipartUpload` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html
#[derive(Debug, Clone)]
pub struct CompleteMultipartUpload<'a, I> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,
    object: &'a str,
    upload_id: &'a str,

    etags: I,
}

impl<'a, I> CompleteMultipartUpload<'a, I> {
    #[inline]
    pub fn new(
        bucket: &'a Bucket,
        credentials: Option<&'a Credentials>,
        object: &'a str,
        upload_id: &'a str,
        etags: I,
    ) -> Self {
        Self {
            bucket,
            credentials,
            object,

            upload_id,
            etags,
        }
    }
}

impl<'a, I> CompleteMultipartUpload<'a, I>
where
    I: Iterator<Item = &'a str>,
{
    fn sign_with_time(&self, expires_in: Duration, time: &OffsetDateTime) -> Url {
        let url = self.bucket.object_url(self.object).unwrap();
        let query = iter::once(("uploadId", self.upload_id));

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

    pub fn body(self) -> String {
        #[derive(Serialize)]
        #[serde(rename = "CompleteMultipartUpload")]
        struct CompleteMultipartUploadSerde<'a> {
            #[serde(rename = "Part")]
            parts: Vec<Part<'a>>,
        }

        #[derive(Serialize)]
        struct Part<'a> {
            #[serde(rename = "$value")]
            nodes: Vec<Node<'a>>,
        }

        #[derive(Serialize)]
        enum Node<'a> {
            ETag(&'a str),
            PartNumber(u16),
        }

        let parts = self
            .etags
            .enumerate()
            .map(|(i, etag)| Part {
                nodes: vec![Node::ETag(etag), Node::PartNumber(i as u16 + 1)],
            })
            .collect::<Vec<_>>();

        let req = CompleteMultipartUploadSerde { parts };

        quick_xml::se::to_string(&req).unwrap()
    }
}

impl<'a, I> S3Action for CompleteMultipartUpload<'a, I>
where
    I: Iterator<Item = &'a str>,
{
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

        let etags = ["123456789", "abcdef"];
        let action = CompleteMultipartUpload::new(
            &bucket,
            Some(&credentials),
            "test.txt",
            "abcd",
            etags.iter().copied(),
        );

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&uploadId=abcd&X-Amz-Signature=19b9d341ce3c6ebd9f049882e875dcad4adc493d9d46d55148f4113146c53dd8";

        assert_eq!(expected, url.as_str());

        let expected = "<CompleteMultipartUpload><Part><ETag>123456789</ETag><PartNumber>1</PartNumber></Part><Part><ETag>abcdef</ETag><PartNumber>2</PartNumber></Part></CompleteMultipartUpload>";
        assert_eq!(action.body(), expected);
    }

    #[test]
    fn anonymous_custom_query() {
        let expires_in = Duration::from_secs(86400);

        let endpoint = "https://s3.amazonaws.com".parse().unwrap();
        let bucket =
            Bucket::new(endpoint, false, "examplebucket".into(), "us-east-1".into()).unwrap();

        let etags = ["123456789", "abcdef"];
        let action =
            CompleteMultipartUpload::new(&bucket, None, "test.txt", "abcd", etags.iter().copied());
        let url = action.sign(expires_in);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?uploadId=abcd";

        assert_eq!(expected, url.as_str());
    }
}
