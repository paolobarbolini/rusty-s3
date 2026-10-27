use std::iter;
use std::time::Duration;

use instant_xml::FromXml;
use jiff::Timestamp;
use url::Url;

use crate::actions::{Method, S3_XML_NS, S3Action};
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

/// The response of a [`ListParts`] request
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, FromXml)]
#[xml(rename = "ListPartsResult", ns(S3_XML_NS))]
pub struct ListPartsResponse {
    /// The listed parts
    pub parts: Vec<PartsContent>,
    /// The maximum number of parts the response could contain
    #[xml(rename = "MaxParts")]
    pub max_parts: u16,
    #[xml(rename = "IsTruncated")]
    is_truncated: bool,
    /// The part number to continue listing from, if the list is truncated
    #[xml(rename = "NextPartNumberMarker")]
    pub next_part_number_marker: Option<u16>,
}

/// A part listed by [`ListParts`]
#[derive(Debug, Clone, FromXml)]
#[xml(rename = "Part", ns(S3_XML_NS))]
pub struct PartsContent {
    /// The part number of the part
    #[xml(rename = "PartNumber")]
    pub number: u16,
    /// The entity tag of the part
    #[xml(rename = "ETag")]
    pub etag: String,
    /// When the part was uploaded
    #[xml(rename = "LastModified")]
    pub last_modified: String,
    /// The size of the part, in bytes
    #[xml(rename = "Size")]
    pub size: u64,
}

impl<'a> ListParts<'a> {
    /// Construct a new `ListParts` action
    ///
    /// Without `credentials` the request isn't signed.
    #[must_use]
    pub const fn new(
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

    /// Sets the maximum number of parts returned in the response, which is
    /// 1,000 by default.
    pub fn set_max_parts(&mut self, max_parts: u16) {
        self.query.insert("max-parts", max_parts.to_string());
    }

    /// Only list the parts with a part number greater than `part_number_marker`,
    /// like the `next_part_number_marker` of the previous response.
    pub fn set_part_number_marker(&mut self, part_number_marker: u16) {
        self.query
            .insert("part-number-marker", part_number_marker.to_string());
    }

    /// Parse the XML response from S3 into a struct
    ///
    /// The next part number marker is only kept if the response is truncated.
    ///
    /// # Errors
    ///
    /// Will return an error if the XML cannot be deserialized, or if the
    /// response is truncated without a part number to continue listing from
    pub fn parse_response(s: &str) -> Result<ListPartsResponse, instant_xml::Error> {
        let mut parts: ListPartsResponse = instant_xml::from_str(s)?;
        match (parts.is_truncated, parts.next_part_number_marker) {
            (false, _) => parts.next_part_number_marker = None,
            // Part numbers start from 1, so continuing from 0 would start over
            (true, None | Some(0)) => {
                return Err(instant_xml::Error::MissingValue(
                    "ListPartsResponse::next_part_number_marker",
                ));
            }
            (true, Some(_)) => {}
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

    use crate::{Bucket, Credentials, UrlStyle};

    use super::*;

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
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?max-parts=50&part-number-marker=100&uploadId=abcd";
        assert_eq!(expected, url.as_str());
    }

    #[test]
    fn parse() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
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
        assert_eq!(part_1.size, 10_485_760);

        let part_2 = &parsed.parts[1];
        assert_eq!(part_2.etag, "\"aaaa18db4cc2f85cedef654fccc4a4x8\"");
        assert_eq!(part_2.number, 3);
        assert_eq!(part_2.last_modified, "2010-11-10T20:48:33.000Z");
        assert_eq!(part_2.size, 10_485_760);

        assert_eq!(parsed.max_parts, 2);
        assert_eq!(parsed.next_part_number_marker, Some(3));
    }

    #[test]
    fn parse_no_parts() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
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

    /// A page of a listing, with `marker` before its `MaxParts`
    fn page(is_truncated: bool, marker: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListPartsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
          <Bucket>example-bucket</Bucket>
          <Key>example-object</Key>
          <UploadId>XXBsb2FkIElEIGZvciBlbHZpbmcncyVcdS1tb3ZpZS5tMnRzEEEwbG9hZA</UploadId>
          <PartNumberMarker>2</PartNumberMarker>
          {marker}
          <MaxParts>1</MaxParts>
          <IsTruncated>{is_truncated}</IsTruncated>
          <Part>
            <PartNumber>3</PartNumber>
            <LastModified>2010-11-10T20:48:33.000Z</LastModified>
            <ETag>"aaaa18db4cc2f85cedef654fccc4a4x8"</ETag>
            <Size>10485760</Size>
          </Part>
        </ListPartsResult>
        "#
        )
    }

    #[test]
    fn parse_truncated_without_next_part_number_marker() {
        for marker in [
            "",
            "<NextPartNumberMarker></NextPartNumberMarker>",
            "<NextPartNumberMarker>0</NextPartNumberMarker>",
        ] {
            let err = ListParts::parse_response(&page(true, marker)).unwrap_err();
            assert_eq!(
                err,
                instant_xml::Error::MissingValue("ListPartsResponse::next_part_number_marker"),
                "{marker}"
            );
        }
    }

    #[test]
    fn parse_not_truncated_ignores_next_part_number_marker() {
        // On the last page MinIO sends 0, and versitygw the last part number
        for marker in ["0", "3"] {
            let marker = format!("<NextPartNumberMarker>{marker}</NextPartNumberMarker>");
            let parsed = ListParts::parse_response(&page(false, &marker)).unwrap();
            assert_eq!(parsed.parts.len(), 1);
            assert!(parsed.next_part_number_marker.is_none(), "{marker}");
        }
    }
}
