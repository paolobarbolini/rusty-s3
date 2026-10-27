use std::iter;
use std::time::Duration;

use instant_xml::ToXml;
use jiff::Timestamp;
use url::Url;

use crate::actions::Method;
use crate::actions::S3Action;
use crate::signing::sign;
use crate::sorting_iter::SortingIterator;
use crate::{Bucket, Credentials, Map};

/// Complete a multipart upload.
///
/// Find out more about `CompleteMultipartUpload` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct CompleteMultipartUpload<'a, I> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,
    object: &'a str,
    upload_id: &'a str,

    etags: I,

    query: Map<'a>,
    headers: Map<'a>,
}

impl<'a, I> CompleteMultipartUpload<'a, I> {
    #[inline]
    pub const fn new(
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

            query: Map::new(),
            headers: Map::new(),
        }
    }
}

impl<'a, I> CompleteMultipartUpload<'a, I>
where
    I: Iterator<Item = &'a str>,
{
    /// Generate the XML body for the request.
    ///
    /// # Panics
    ///
    /// Panics if there are more than [`u16::MAX`] parts.
    pub fn body(self) -> String {
        #[derive(ToXml)]
        #[xml(rename = "CompleteMultipartUpload")]
        struct CompleteMultipartUploadBody<'a> {
            parts: Vec<Part<'a>>,
        }

        #[derive(ToXml)]
        #[xml(rename = "Part")]
        struct Part<'a> {
            #[xml(rename = "ETag")]
            etag: &'a str,
            #[xml(rename = "PartNumber")]
            part_number: u16,
        }

        let parts = self
            .etags
            .enumerate()
            .map(|(i, etag)| Part {
                etag,
                part_number: u16::try_from(i + 1).expect("convert to u16"),
            })
            .collect::<Vec<_>>();

        let req = CompleteMultipartUploadBody { parts };

        instant_xml::to_string(&req).unwrap()
    }
}

impl<'a, I> S3Action<'a> for CompleteMultipartUpload<'a, I>
where
    I: Iterator<Item = &'a str>,
{
    const METHOD: Method = Method::Post;

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
    #[should_panic(expected = "convert to u16")]
    fn too_many_parts() {
        let endpoint = "https://s3.amazonaws.com".parse().unwrap();
        let bucket = Bucket::new(
            endpoint,
            UrlStyle::VirtualHost,
            "examplebucket",
            "us-east-1",
        )
        .unwrap();

        // Part numbers start at 1, so the last ETag's would be 65536
        let etags = iter::repeat_n("abcdef", usize::from(u16::MAX) + 1);
        let action = CompleteMultipartUpload::new(&bucket, None, "test.txt", "abcd", etags);
        let _ = action.body();
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

        let etags = ["123456789", "abcdef"];
        let mut action =
            CompleteMultipartUpload::new(&bucket, None, "test.txt", "abcd", etags.iter().copied());
        action.query_mut().insert("x-id", "CompleteMultipartUpload");

        let url = action.sign(expires_in);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?uploadId=abcd&x-id=CompleteMultipartUpload";

        assert_eq!(expected, url.as_str());
    }
}
