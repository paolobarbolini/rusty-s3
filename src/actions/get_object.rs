use std::time::Duration;

use time::OffsetDateTime;
use url::Url;

use super::S3Action;
use crate::actions::Method;
use crate::signing::sign;
use crate::{Bucket, Credentials, Map};

/// Retrieve an object from S3, using a `GET` request.
///
/// Find out more about `GetObject` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html
#[derive(Debug, Clone)]
pub struct GetObject<'a> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,
    object: &'a str,

    query: Map<'a>,
    headers: Map<'a>,
}

impl<'a> GetObject<'a> {
    #[inline]
    pub fn new(bucket: &'a Bucket, credentials: Option<&'a Credentials>, object: &'a str) -> Self {
        Self {
            bucket,
            credentials,
            object,

            query: Map::new(),
            headers: Map::new(),
        }
    }
}

impl<'a> S3Action<'a> for GetObject<'a> {
    const METHOD: Method = Method::Get;

    fn query_mut(&mut self) -> &mut Map<'a> {
        &mut self.query
    }

    fn headers_mut(&mut self) -> &mut Map<'a> {
        &mut self.headers
    }

    fn sign_with_time(&self, expires_in: Duration, time: &OffsetDateTime) -> Url {
        let url = self.bucket.object_url(self.object).unwrap();

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
                self.query.iter(),
                self.headers.iter(),
            ),
            None => crate::signing::util::add_query_params(url, self.query.iter()),
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

        let action = GetObject::new(&bucket, Some(&credentials), "test.txt");

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&X-Amz-Signature=aeeed9bbccd4d02ee5c0109b86d86835f995330da4c265957d157751f604d404";

        assert_eq!(expected, url.as_str());
    }

    #[test]
    fn aws_example_custom_query() {
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

        let mut action = GetObject::new(&bucket, Some(&credentials), "test.txt");
        action
            .query_mut()
            .insert("response-content-type", "text/plain");

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&response-content-type=text%2Fplain&X-Amz-Signature=9cee3ba363b3a52fed152d18bb250d52a459d0905600d9b032825a3794ffd2cb";

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

        let mut action = GetObject::new(&bucket, None, "test.txt");
        action
            .query_mut()
            .insert("response-content-type", "text/plain");

        let url = action.sign(expires_in);
        let expected =
            "https://examplebucket.s3.amazonaws.com/test.txt?response-content-type=text%2Fplain";

        assert_eq!(expected, url.as_str());
    }
}
