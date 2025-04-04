use std::borrow::Cow;
use std::io::{BufReader, Read};
use std::time::Duration;

use jiff::Timestamp;
use serde::Deserialize;
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
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct ListObjectsV2<'a> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,

    query: Map<'a>,
    headers: Map<'a>,
}

#[allow(clippy::module_name_repetitions)]
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
    pub max_keys: Option<u16>,
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
    pub storage_class: Option<String>,
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
    #[must_use]
    pub fn new(bucket: &'a Bucket, credentials: Option<&'a Credentials>) -> Self {
        let mut query = Map::new();
        query.insert("list-type", "2");
        query.insert("encoding-type", "url");

        Self {
            bucket,
            credentials,

            query,
            headers: Map::new(),
        }
    }

    /// Limits the response to keys that begin with the specified prefix.
    ///
    /// See <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html#API_ListObjectsV2_RequestSyntax> for more infos.
    /// # Example
    /// ```
    /// # let bucket = rusty_s3::Bucket::new(url::Url::parse("http://rusty_s3/").unwrap(), rusty_s3::UrlStyle::Path, "doggo", "doggoland").unwrap();
    /// let mut list = bucket.list_objects_v2(None);
    /// list.with_prefix("tamo");
    /// ```
    pub fn with_prefix(&mut self, prefix: impl Into<Cow<'a, str>>) {
        self.query_mut().insert("prefix", prefix);
    }

    /// A delimiter is a character that you use to group keys.
    ///
    /// See <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html#API_ListObjectsV2_RequestSyntax> for more infos.
    /// # Example
    /// ```
    /// # let bucket = rusty_s3::Bucket::new(url::Url::parse("http://rusty_s3/").unwrap(), rusty_s3::UrlStyle::Path, "doggo", "doggoland").unwrap();
    /// let mut list = bucket.list_objects_v2(None);
    /// list.with_delimiter("/");
    /// ```
    pub fn with_delimiter(&mut self, delimiter: impl Into<Cow<'a, str>>) {
        self.query_mut().insert("delimiter", delimiter);
    }

    /// `StartAfter` is where you want Amazon S3 to start listing from.
    /// Amazon S3 starts listing after this specified key.
    /// `StartAfter` can be any key in the bucket.
    ///
    /// See <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html#API_ListObjectsV2_RequestSyntax> for more infos.
    /// # Example
    /// ```
    /// # let bucket = rusty_s3::Bucket::new(url::Url::parse("http://rusty_s3/").unwrap(), rusty_s3::UrlStyle::Path, "doggo", "doggoland").unwrap();
    /// let mut list = bucket.list_objects_v2(None);
    /// list.with_start_after("tamo"); // <- This token should come from a previous call to the list API.
    /// ```
    pub fn with_start_after(&mut self, start_after: impl Into<Cow<'a, str>>) {
        self.query_mut().insert("start-after", start_after);
    }

    /// `ContinuationToken` indicates to Amazon S3 that the list is being continued on this bucket with a token.
    /// `ContinuationToken` is obfuscated and is not a real key.
    ///
    /// See <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html#API_ListObjectsV2_RequestSyntax> for more infos.
    /// # Example
    /// ```
    /// # let bucket = rusty_s3::Bucket::new(url::Url::parse("http://rusty_s3/").unwrap(), rusty_s3::UrlStyle::Path, "doggo", "doggoland").unwrap();
    /// let mut list = bucket.list_objects_v2(None);
    /// list.with_continuation_token("tamo"); // <- This token should come from a previous call to the list API.
    /// ```
    pub fn with_continuation_token(&mut self, continuation_token: impl Into<Cow<'a, str>>) {
        self.query_mut()
            .insert("continuation-token", continuation_token);
    }

    /// Sets the maximum number of keys returned in the response.
    /// By default, the action returns up to 1,000 key names.
    /// The response might contain fewer keys but will never contain more.
    ///
    /// See <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html#API_ListObjectsV2_RequestSyntax> for more infos.
    /// # Example
    /// ```
    /// # let bucket = rusty_s3::Bucket::new(url::Url::parse("http://rusty_s3/").unwrap(), rusty_s3::UrlStyle::Path, "doggo", "doggoland").unwrap();
    /// let mut list = bucket.list_objects_v2(None);
    /// list.with_continuation_token("tamo"); // <- This token should come from a previous call to the list API.
    /// ```
    pub fn with_max_keys(&mut self, max_keys: usize) {
        self.query_mut().insert("max-keys", max_keys.to_string());
    }

    /// Parse the XML response from S3 into a struct.
    ///
    /// # Errors
    ///
    /// Returns an error if the XML response could not be parsed.
    pub fn parse_response(
        s: impl AsRef<[u8]>,
    ) -> Result<ListObjectsV2Response, quick_xml::DeError> {
        Self::parse_response_from_reader(&mut s.as_ref())
    }

    /// Parse the XML response from S3 into a struct.
    ///
    /// # Errors
    ///
    /// Returns an error if the XML response could not be parsed.
    pub fn parse_response_from_reader(
        s: impl Read,
    ) -> Result<ListObjectsV2Response, quick_xml::DeError> {
        let mut parsed: ListObjectsV2Response = quick_xml::de::from_reader(BufReader::new(s))?;

        // S3 returns an Owner with an empty DisplayName and ID when fetch-owner is disabled
        for content in &mut parsed.contents {
            if let Some(owner) = &content.owner {
                if owner.id.is_empty() && owner.display_name.is_empty() {
                    content.owner = None;
                }
            }
        }

        Ok(parsed)
    }
}

impl<'a> S3Action<'a> for ListObjectsV2<'a> {
    const METHOD: Method = Method::Get;

    fn query_mut(&mut self) -> &mut Map<'a> {
        &mut self.query
    }

    fn headers_mut(&mut self) -> &mut Map<'a> {
        &mut self.headers
    }

    fn sign_with_time(&self, expires_in: Duration, time: &Timestamp) -> Url {
        let url = self.bucket.base_url().clone();

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
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{Bucket, Credentials, UrlStyle};

    #[test]
    fn aws_example() {
        // Fri, 24 May 2013 00:00:00 GMT
        let date = Timestamp::from_second(1369353600).unwrap();
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

        let action = ListObjectsV2::new(&bucket, Some(&credentials));

        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&encoding-type=url&list-type=2&X-Amz-Signature=58e7f65928710f045f6a7e1f7a32b3426b4895900fad799db66faa3ff8b18bd5";

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
        assert_eq!(item_1.storage_class, Some("STANDARD".to_string()));

        let item_2 = &parsed.contents[1];
        assert_eq!(item_2.etag, "\"5927c5d64d94a5786f90003aa26d0159-1\"");
        assert_eq!(item_2.key, "idk.txt");
        assert_eq!(item_2.last_modified, "2020-12-05T08:23:52.215Z");
        assert!(item_2.owner.is_none());
        assert_eq!(item_2.size, 9);
        assert_eq!(item_2.storage_class, Some("STANDARD".to_string()));

        let item_3 = &parsed.contents[2];
        assert_eq!(item_3.etag, "\"f7dbec93a0932ccb4d0f4e512eb1a443\"");
        assert_eq!(item_3.key, "img.jpg");
        assert_eq!(item_3.last_modified, "2020-11-26T20:21:35.858Z");
        assert!(item_3.owner.is_none());
        assert_eq!(item_3.size, 41259);
        assert_eq!(item_3.storage_class, Some("STANDARD".to_string()));

        assert_eq!(parsed.max_keys, Some(4500));
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

        assert_eq!(parsed.max_keys, Some(4500));
        assert!(parsed.common_prefixes.is_empty());
        assert!(parsed.next_continuation_token.is_none());
        assert!(parsed.start_after.is_none());
    }
}
