use std::iter;
use std::time::Duration;

use serde::Deserialize;
use time::OffsetDateTime;
use url::Url;

use crate::actions::Method;
use crate::actions::S3Action;
use crate::signing::sign;
use crate::{Bucket, Credentials, Map};

/// List all objects in the bucket.
///
/// If `next_continuation_token` is `Some` the response is truncated, and the
/// rest of the list can be retrieved by reusing the `ListObjectV2` action
/// but with `continuation-token` set to the value of `next_continuation_token`
/// received in the previous response.
///
/// Find out more about `ListObjectsV2` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html
#[derive(Debug, Clone)]
pub struct ListObjectsV2<'a> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,

    query: Map<'a>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListObjectsV2Response {
    // #[serde(rename = "IsTruncated")]
    // is_truncated: bool,
    #[serde(rename = "Contents")]
    #[serde(default)]
    pub contents: Vec<ListObjectsContent>,

    // #[serde(rename = "Name")]
    // name: String,
    // #[serde(rename = "Prefix")]
    // prefix: String,
    // #[serde(rename = "Delimiter")]
    // delimiter: String,
    #[serde(rename = "MaxKeys")]
    pub max_keys: u16,
    #[serde(rename = "CommonPrefixes", default)]
    pub common_prefixes: Vec<CommonPrefixes>,
    // #[serde(rename = "EncodingType")]
    // encoding_type: String,
    // #[serde(rename = "KeyCount")]
    // key_count: u16,
    // #[serde(rename = "ContinuationToken")]
    // continuation_token: Option<String>,
    #[serde(rename = "NextContinuationToken")]
    pub next_continuation_token: Option<String>,
    #[serde(rename = "StartAfter")]
    pub start_after: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListObjectsContent {
    #[serde(rename = "ETag")]
    pub etag: String,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "LastModified")]
    pub last_modified: String,
    #[serde(rename = "Owner")]
    pub owner: Option<ListObjectsOwner>,
    #[serde(rename = "Size")]
    pub size: u64,
    #[serde(rename = "StorageClass")]
    pub storage_class: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListObjectsOwner {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "DisplayName")]
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommonPrefixes {
    #[serde(rename = "Prefix")]
    pub prefix: String,
}

impl<'a> ListObjectsV2<'a> {
    pub fn new(bucket: &'a Bucket, credentials: Option<&'a Credentials>) -> Self {
        let mut query = Map::new();
        query.insert("list-type", "2");
        query.insert("encoding-type", "url");

        Self {
            bucket,
            credentials,

            query,
        }
    }

    pub fn query_mut(&mut self) -> &mut Map<'a> {
        &mut self.query
    }

    pub fn parse_response(s: &str) -> Result<ListObjectsV2Response, quick_xml::DeError> {
        let mut parsed: ListObjectsV2Response = quick_xml::de::from_str(s)?;

        // S3 returns an Owner with an empty DisplayName and ID when fetch-owner is disabled
        for content in parsed.contents.iter_mut() {
            if let Some(owner) = &content.owner {
                if owner.id.is_empty() && owner.display_name.is_empty() {
                    content.owner = None;
                }
            }
        }

        Ok(parsed)
    }

    fn sign_with_time(&self, expires_in: Duration, time: &OffsetDateTime) -> Url {
        let url = self.bucket.base_url().clone();

        match self.credentials {
            Some(credentials) => sign(
                time,
                Method::Get,
                url,
                credentials.key(),
                credentials.secret(),
                credentials.token(),
                self.bucket.region(),
                expires_in.as_secs(),
                self.query.iter(),
                iter::empty(),
            ),
            None => crate::signing::util::add_query_params(url, self.query.iter()),
        }
    }
}

impl<'a> S3Action for ListObjectsV2<'a> {
    const METHOD: Method = Method::Get;

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

        let action = ListObjectsV2::new(&bucket, Some(&credentials));

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&encoding-type=url&list-type=2&X-Amz-Signature=58e7f65928710f045f6a7e1f7a32b3426b4895900fad799db66faa3ff8b18bd5";

        assert_eq!(expected, url.as_str());
    }

    #[test]
    fn anonymous_custom_query() {
        let expires_in = Duration::from_secs(86400);

        let endpoint = "https://s3.amazonaws.com".parse().unwrap();
        let bucket =
            Bucket::new(endpoint, false, "examplebucket".into(), "us-east-1".into()).unwrap();

        let mut action = ListObjectsV2::new(&bucket, None);
        action.query_mut().insert("continuation-token", "duck");

        let url = action.sign(expires_in);
        let expected = "https://examplebucket.s3.amazonaws.com/?continuation-token=duck&encoding-type=url&list-type=2";

        assert_eq!(expected, url.as_str());
    }

    #[test]
    fn parse() {
        let input = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
            <Name>test</Name>
            <Prefix></Prefix>
            <KeyCount>3</KeyCount>
            <MaxKeys>4500</MaxKeys>
            <Delimiter></Delimiter>
            <IsTruncated>false</IsTruncated>
            <Contents>
                <Key>duck.jpg</Key>
                <LastModified>2020-12-01T20:43:11.794Z</LastModified>
                <ETag>"bfd537a51d15208163231b0711e0b1f3"</ETag>
                <Size>4274</Size>
                <Owner>
                    <ID></ID>
                    <DisplayName></DisplayName>
                </Owner>
                <StorageClass>STANDARD</StorageClass>
            </Contents>
            <Contents>
                <Key>idk.txt</Key>
                <LastModified>2020-12-05T08:23:52.215Z</LastModified>
                <ETag>"5927c5d64d94a5786f90003aa26d0159-1"</ETag>
                <Size>9</Size>
                <Owner>
                    <ID></ID>
                    <DisplayName></DisplayName>
                </Owner>
                <StorageClass>STANDARD</StorageClass>
            </Contents>
            <Contents>
                <Key>img.jpg</Key>
                <LastModified>2020-11-26T20:21:35.858Z</LastModified>
                <ETag>"f7dbec93a0932ccb4d0f4e512eb1a443"</ETag>
                <Size>41259</Size>
                <Owner>
                    <ID></ID>
                    <DisplayName></DisplayName>
                </Owner>
                <StorageClass>STANDARD</StorageClass>
            </Contents>
            <EncodingType>url</EncodingType>
        </ListBucketResult>
        "#;

        let parsed = ListObjectsV2::parse_response(input).unwrap();
        assert_eq!(parsed.contents.len(), 3);

        let item_1 = &parsed.contents[0];
        assert_eq!(item_1.etag, "\"bfd537a51d15208163231b0711e0b1f3\"");
        assert_eq!(item_1.key, "duck.jpg");
        assert_eq!(item_1.last_modified, "2020-12-01T20:43:11.794Z");
        assert!(item_1.owner.is_none());
        assert_eq!(item_1.size, 4274);
        assert_eq!(item_1.storage_class, "STANDARD");

        let item_2 = &parsed.contents[1];
        assert_eq!(item_2.etag, "\"5927c5d64d94a5786f90003aa26d0159-1\"");
        assert_eq!(item_2.key, "idk.txt");
        assert_eq!(item_2.last_modified, "2020-12-05T08:23:52.215Z");
        assert!(item_2.owner.is_none());
        assert_eq!(item_2.size, 9);
        assert_eq!(item_2.storage_class, "STANDARD");

        let item_3 = &parsed.contents[2];
        assert_eq!(item_3.etag, "\"f7dbec93a0932ccb4d0f4e512eb1a443\"");
        assert_eq!(item_3.key, "img.jpg");
        assert_eq!(item_3.last_modified, "2020-11-26T20:21:35.858Z");
        assert!(item_3.owner.is_none());
        assert_eq!(item_3.size, 41259);
        assert_eq!(item_3.storage_class, "STANDARD");

        assert_eq!(parsed.max_keys, 4500);
        assert!(parsed.common_prefixes.is_empty());
        assert!(parsed.next_continuation_token.is_none());
        assert!(parsed.start_after.is_none());
    }

    #[test]
    fn parse_no_contents() {
        let input = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
            <Name>test</Name>
            <Prefix></Prefix>
            <KeyCount>0</KeyCount>
            <MaxKeys>4500</MaxKeys>
            <Delimiter></Delimiter>
            <IsTruncated>false</IsTruncated>
            <EncodingType>url</EncodingType>
        </ListBucketResult>
        "#;

        let parsed = ListObjectsV2::parse_response(input).unwrap();
        assert_eq!(parsed.contents.is_empty(), true);

        assert_eq!(parsed.max_keys, 4500);
        assert!(parsed.common_prefixes.is_empty());
        assert!(parsed.next_continuation_token.is_none());
        assert!(parsed.start_after.is_none());
    }
}
