use std::iter;
use std::time::Duration;

use serde::Deserialize;
use time::OffsetDateTime;
use url::Url;

use crate::actions::Method;
use crate::actions::S3Action;
use crate::signing::sign;
use crate::sorting_iter::SortingIterator;
use crate::{Bucket, Credentials, Map};

/// Lists the parts that have been uploaded for a specific multipart upload.
///
/// If `next_part_number_marker` is `Some` the response is truncated, and the
/// rest of the list can be retrieved by reusing the `ListParts` action
/// but with `part_number_marker` set to the value of `next_part_number_marker`
/// received in the previous response.
///
/// Find out more about `ListParts` from the [AWS API Reference][api]
///
/// [api]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListParts.html
#[derive(Debug, Clone)]
pub struct ListParts<'a> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,
    object: &'a str,
    upload_id: &'a str,

    query: Map<'a>,
    headers: Map<'a>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListPartsResponse {
    #[serde(rename = "Part")]
    #[serde(default)]
    pub parts: Vec<PartsContent>,
    #[serde(rename = "MaxParts")]
    pub max_parts: u16,
    #[serde(rename = "IsTruncated")]
    is_truncated: bool,
    #[serde(rename = "NextPartNumberMarker")]
    pub next_part_number_marker: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PartsContent {
    #[serde(rename = "PartNumber")]
    pub number: u16,
    #[serde(rename = "ETag")]
    pub etag: String,
    #[serde(rename = "LastModified")]
    pub last_modified: String,
    #[serde(rename = "Size")]
    pub size: u64,
}

impl<'a> ListParts<'a> {
    pub fn new(
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

    pub fn set_max_parts(&mut self, max_parts: u16) {
        self.query.insert("max-parts", max_parts.to_string());
    }

    pub fn set_part_number_marker(&mut self, part_number_marker: u16) {
        self.query
            .insert("part-number-marker", part_number_marker.to_string());
    }

    pub fn parse_response(s: &str) -> Result<ListPartsResponse, quick_xml::DeError> {
        let mut parts: ListPartsResponse = quick_xml::de::from_str(s)?;
        if !parts.is_truncated {
            parts.next_part_number_marker = None;
        }
        Ok(parts)
    }
}

impl<'a> S3Action<'a> for ListParts<'a> {
    const METHOD: Method = Method::Get;

    fn query_mut(&mut self) -> &mut Map<'a> {
        &mut self.query
    }

    fn headers_mut(&mut self) -> &mut Map<'a> {
        &mut self.headers
    }

    fn sign_with_time(&self, expires_in: Duration, time: &OffsetDateTime) -> Url {
        let url = self.bucket.object_url(self.object).unwrap();
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

        let mut action = ListParts::new(&bucket, Some(&credentials), "test.txt", "abcd");
        action.set_max_parts(100);
        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&max-parts=100&uploadId=abcd&X-Amz-Signature=10a814258808a79054a80e2aff66e95faba686648eb50bd143fe7fe7d6d7b6ce";
        assert_eq!(expected, url.as_str());

        let mut action = ListParts::new(&bucket, Some(&credentials), "test.txt", "abcd");
        action.set_max_parts(50);
        action.set_part_number_marker(100);
        let url = action.sign_with_time(expires_in, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&max-parts=50&part-number-marker=100&uploadId=abcd&X-Amz-Signature=ea8eecb4f2534d606474497e6088ceb262081bf7c5a289ff0598aafdd66055da";
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

        let mut action = ListParts::new(&bucket, None, "test.txt", "abcd");
        action.set_max_parts(100);
        let url = action.sign(expires_in);
        let expected =
            "https://examplebucket.s3.amazonaws.com/test.txt?max-parts=100&uploadId=abcd";
        assert_eq!(expected, url.as_str());

        let mut action = ListParts::new(&bucket, None, "test.txt", "abcd");
        action.set_max_parts(50);
        action.set_part_number_marker(100);
        let url = action.sign(expires_in);
        let expected =
            "https://examplebucket.s3.amazonaws.com/test.txt?max-parts=50&part-number-marker=100&uploadId=abcd";
        assert_eq!(expected, url.as_str());
    }

    #[test]
    fn parse() {
        let input = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <ListPartsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
          <Bucket>example-bucket</Bucket>
          <Key>example-object</Key>
          <UploadId>XXBsb2FkIElEIGZvciBlbHZpbmcncyVcdS1tb3ZpZS5tMnRzEEEwbG9hZA</UploadId>
          <Initiator>
              <ID>arn:aws:iam::111122223333:user/some-user-11116a31-17b5-4fb7-9df5-b288870f11xx</ID>
              <DisplayName>umat-user-11116a31-17b5-4fb7-9df5-b288870f11xx</DisplayName>
          </Initiator>
          <Owner>
            <ID>75aa57f09aa0c8caeab4f8c24e99d10f8e7faeebf76c078efc7c6caea54ba06a</ID>
            <DisplayName>someName</DisplayName>
          </Owner>
          <StorageClass>STANDARD</StorageClass>
          <PartNumberMarker>1</PartNumberMarker>
          <NextPartNumberMarker>3</NextPartNumberMarker>
          <MaxParts>2</MaxParts>
          <IsTruncated>true</IsTruncated>
          <Part>
            <PartNumber>2</PartNumber>
            <LastModified>2010-11-10T20:48:34.000Z</LastModified>
            <ETag>"7778aef83f66abc1fa1e8477f296d394"</ETag>
            <Size>10485760</Size>
          </Part>
          <Part>
            <PartNumber>3</PartNumber>
            <LastModified>2010-11-10T20:48:33.000Z</LastModified>
            <ETag>"aaaa18db4cc2f85cedef654fccc4a4x8"</ETag>
            <Size>10485760</Size>
          </Part>
        </ListPartsResult>
        "#;

        let parsed = ListParts::parse_response(input).unwrap();
        assert_eq!(parsed.parts.len(), 2);

        let part_1 = &parsed.parts[0];
        assert_eq!(part_1.etag, "\"7778aef83f66abc1fa1e8477f296d394\"");
        assert_eq!(part_1.number, 2);
        assert_eq!(part_1.last_modified, "2010-11-10T20:48:34.000Z");
        assert_eq!(part_1.size, 10485760);

        let part_2 = &parsed.parts[1];
        assert_eq!(part_2.etag, "\"aaaa18db4cc2f85cedef654fccc4a4x8\"");
        assert_eq!(part_2.number, 3);
        assert_eq!(part_2.last_modified, "2010-11-10T20:48:33.000Z");
        assert_eq!(part_2.size, 10485760);

        assert_eq!(parsed.max_parts, 2);
        assert_eq!(parsed.next_part_number_marker, Some(3));
    }

    #[test]
    fn parse_no_parts() {
        let input = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <ListPartsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
          <Bucket>example-bucket</Bucket>
          <Key>example-object</Key>
          <UploadId>XXBsb2FkIElEIGZvciBlbHZpbmcncyVcdS1tb3ZpZS5tMnRzEEEwbG9hZA</UploadId>
          <Initiator>
              <ID>arn:aws:iam::111122223333:user/some-user-11116a31-17b5-4fb7-9df5-b288870f11xx</ID>
              <DisplayName>umat-user-11116a31-17b5-4fb7-9df5-b288870f11xx</DisplayName>
          </Initiator>
          <Owner>
            <ID>75aa57f09aa0c8caeab4f8c24e99d10f8e7faeebf76c078efc7c6caea54ba06a</ID>
            <DisplayName>someName</DisplayName>
          </Owner>
          <StorageClass>STANDARD</StorageClass>
          <PartNumberMarker>1</PartNumberMarker>
          <MaxParts>2</MaxParts>
          <IsTruncated>false</IsTruncated>
        </ListPartsResult>
        "#;

        let parsed = ListParts::parse_response(input).unwrap();
        assert!(parsed.parts.is_empty());
        assert_eq!(parsed.max_parts, 2);
        assert!(parsed.next_part_number_marker.is_none());
    }
}
