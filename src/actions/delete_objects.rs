use std::iter;
use std::time::Duration;

use md5::{Digest, Md5};
use serde::Serialize;
use time::OffsetDateTime;
use url::Url;

use crate::actions::Method;
use crate::actions::S3Action;
use crate::signing::sign;
use crate::sorting_iter::SortingIterator;
use crate::{Bucket, Credentials, Map};

/// Delete multiple objects from a bucket using a single `POST` request.
///
/// Find out more about `DeleteObjects` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjects.html
#[derive(Debug, Clone)]
pub struct DeleteObjects<'a, I> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,
    objects: I,
    quiet: bool,

    query: Map<'a>,
    headers: Map<'a>,
}

impl<'a, I> DeleteObjects<'a, I> {
    #[inline]
    pub fn new(bucket: &'a Bucket, credentials: Option<&'a Credentials>, objects: I) -> Self {
        Self {
            bucket,
            credentials,
            objects,
            quiet: false,
            query: Map::new(),
            headers: Map::new(),
        }
    }

    pub fn quiet(&self) -> bool {
        self.quiet
    }

    pub fn set_quiet(&mut self, quiet: bool) {
        self.quiet = quiet;
    }
}

#[derive(Debug, Clone, Default)]
pub struct ObjectIdentifier {
    pub key: String,
    pub version_id: Option<String>,
}

impl ObjectIdentifier {
    pub fn new(key: String) -> Self {
        Self {
            key,
            ..Default::default()
        }
    }
}

impl<'a, I> DeleteObjects<'a, I>
where
    I: Iterator<Item = &'a ObjectIdentifier>,
{
    pub fn body_with_md5(self) -> (String, String) {
        #[derive(Serialize)]
        #[serde(rename = "Delete")]
        struct DeleteSerde<'a> {
            #[serde(rename = "Object")]
            objects: Vec<Object<'a>>,
            #[serde(rename = "Quiet")]
            quiet: Option<bool>,
        }
        #[derive(Serialize)]
        #[serde(rename = "Delete")]
        struct Object<'a> {
            #[serde(rename = "$value")]
            nodes: Vec<Node<'a>>,
        }

        #[derive(Serialize)]
        enum Node<'a> {
            Key(&'a str),
            VersionId(&'a str),
        }

        let objects: Vec<Object<'a>> = self
            .objects
            .map(|o| {
                let mut nodes = vec![Node::Key(o.key.as_str())];
                if let Some(ref version_id) = o.version_id {
                    nodes.push(Node::VersionId(version_id.as_str()));
                }
                Object { nodes }
            })
            .collect();

        let req = DeleteSerde {
            objects,
            quiet: self.quiet.then_some(true),
        };

        let body = quick_xml::se::to_string(&req).unwrap();

        let content_md5 = crate::base64::encode(Md5::digest(body.as_bytes()));
        (body, content_md5)
    }
}

impl<'a, I> S3Action<'a> for DeleteObjects<'a, I>
where
    I: Iterator<Item = &'a ObjectIdentifier>,
{
    const METHOD: Method = Method::Post;

    fn query_mut(&mut self) -> &mut Map<'a> {
        &mut self.query
    }

    fn headers_mut(&mut self) -> &mut Map<'a> {
        &mut self.headers
    }

    fn sign_with_time(&self, expires_in: Duration, time: &OffsetDateTime) -> Url {
        let url = self.bucket.base_url().clone();
        let query = SortingIterator::new(iter::once(("delete", "1")), self.query.iter());

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
    use time::OffsetDateTime;

    use crate::{Bucket, Credentials, UrlStyle};

    use super::*;

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

        let objects = [
            ObjectIdentifier {
                key: "123".to_owned(),
                ..Default::default()
            },
            ObjectIdentifier {
                key: "456".to_owned(),
                version_id: Some("ver1234".to_owned()),
            },
        ];
        let action = DeleteObjects::new(&bucket, Some(&credentials), objects.iter());

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&delete=1&X-Amz-Signature=0e6170ba8cb7873da76b7fb63638658607f484265935099b3d8cea5195af843c";

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

        let objects = [
            ObjectIdentifier {
                key: "123".to_owned(),
                ..Default::default()
            },
            ObjectIdentifier {
                key: "456".to_owned(),
                version_id: Some("ver1234".to_owned()),
            },
        ];
        let action = DeleteObjects::new(&bucket, None, objects.iter());
        let url = action.sign(expires_in);
        let expected = "https://examplebucket.s3.amazonaws.com/?delete=1";

        assert_eq!(expected, url.as_str());
    }
}
