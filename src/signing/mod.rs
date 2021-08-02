use std::{iter, str};

use time::OffsetDateTime;
use url::Url;

use crate::sorting_iter::SortingIterator;
use crate::time_::{ISO8601, YYYYMMDD};
use crate::Method;

mod canonical_request;
mod signature;
mod string_to_sign;
pub(crate) mod util;

#[allow(clippy::too_many_arguments)]
pub fn sign<'a, Q, H>(
    date: &OffsetDateTime,
    method: Method,
    mut url: Url,
    key: &str,
    secret: &str,
    token: Option<&str>,
    region: &str,
    expires_seconds: u64,

    query_string: Q,
    headers: H,
) -> Url
where
    Q: Iterator<Item = (&'a str, &'a str)> + Clone,
    H: Iterator<Item = (&'a str, &'a str)> + Clone,
{
    // Convert `&'a str` into `&str`, in order to later be able to join them to
    // the inner iterators, which because of the references they take to the inner
    // `String`s, have a shorter lifetime than 'a.
    // Thanks to: https://t.me/rustlang_it/61993
    let query_string = query_string.map(|(key, value)| (key, value));
    let headers = headers.map(|(key, value)| (key, value));

    let yyyymmdd = date.format(&YYYYMMDD).expect("invalid format");

    let credential = format!(
        "{}/{}/{}/{}/{}",
        key, yyyymmdd, region, "s3", "aws4_request"
    );
    let date_str = date.format(&ISO8601).expect("invalid format");
    let expires_seconds_string = expires_seconds.to_string();

    let host = url.host_str().expect("host is known");
    let host_header = match (url.scheme(), url.port()) {
        ("http", None) | ("http", Some(80)) | ("https", None) | ("https", Some(443)) => {
            host.to_string()
        }
        ("http", Some(port)) | ("https", Some(port)) => {
            format!("{}:{}", host, port)
        }
        _ => panic!("unsupported url scheme"),
    };

    let standard_headers = iter::once(("host", host_header.as_str()));
    let headers = SortingIterator::new(standard_headers, headers);

    let signed_headers = headers.clone().map(|(key, _)| key);
    let mut signed_headers_str = String::new();
    for header in signed_headers.clone() {
        if !signed_headers_str.is_empty() {
            signed_headers_str.push(';');
        }
        signed_headers_str.push_str(header);
    }

    let a1;
    let a2;
    let standard_query = match token {
        Some(token) => {
            a1 = [
                ("X-Amz-Algorithm", "AWS4-HMAC-SHA256"),
                ("X-Amz-Credential", credential.as_str()),
                ("X-Amz-Date", date_str.as_str()),
                ("X-Amz-Expires", expires_seconds_string.as_str()),
                ("X-Amz-Security-Token", token),
                ("X-Amz-SignedHeaders", &signed_headers_str),
            ];
            a1.iter()
        }
        None => {
            a2 = [
                ("X-Amz-Algorithm", "AWS4-HMAC-SHA256"),
                ("X-Amz-Credential", credential.as_str()),
                ("X-Amz-Date", date_str.as_str()),
                ("X-Amz-Expires", expires_seconds_string.as_str()),
                ("X-Amz-SignedHeaders", &signed_headers_str),
            ];
            a2.iter()
        }
    };

    let query_string = SortingIterator::new(standard_query.copied(), query_string);

    {
        let mut query_pairs = url.query_pairs_mut();
        query_pairs.clear();

        query_pairs.extend_pairs(query_string.clone());
    }

    let canonical_req =
        canonical_request::canonical_request(method, &url, query_string, headers, signed_headers);
    let signed_string = string_to_sign::string_to_sign(date, region, &canonical_req);
    let signature = signature::signature(date, secret, region, &signed_string);

    url.query_pairs_mut()
        .append_pair("X-Amz-Signature", &signature);
    url
}

#[cfg(test)]
mod tests {
    use std::iter;

    use pretty_assertions::assert_eq;
    use time::OffsetDateTime;

    use super::Method;
    use super::*;

    #[test]
    fn aws_example() {
        // Fri, 24 May 2013 00:00:00 GMT
        let date = OffsetDateTime::from_unix_timestamp(1369353600).unwrap();

        let method = Method::Get;
        let url = "https://examplebucket.s3.amazonaws.com/test.txt"
            .parse()
            .unwrap();
        let key = "AKIAIOSFODNN7EXAMPLE";
        let secret = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        let region = "us-east-1";
        let expires_seconds = 86400;

        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&X-Amz-Signature=aeeed9bbccd4d02ee5c0109b86d86835f995330da4c265957d157751f604d404";

        let got = sign(
            &date,
            method,
            url,
            key,
            secret,
            None,
            region,
            expires_seconds,
            iter::empty(),
            iter::empty(),
        );

        assert_eq!(expected, got.as_str());
    }

    #[test]
    fn aws_example_token() {
        // Fri, 24 May 2013 00:00:00 GMT
        let date = OffsetDateTime::from_unix_timestamp(1369353600).unwrap();

        let method = Method::Get;
        let url = "https://examplebucket.s3.amazonaws.com/test.txt"
            .parse()
            .unwrap();
        let key = "AKIAIOSFODNN7EXAMPLE";
        let secret = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        let token = "oej5cie4uctureturdtuc5dctd";
        let region = "us-east-1";
        let expires_seconds = 86400;

        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-Security-Token=oej5cie4uctureturdtuc5dctd&X-Amz-SignedHeaders=host&X-Amz-Signature=bf77b83a7135594046c90a7e7e10cf1a4c8f8ecc1d541d0f42bea6b7670870c7";

        let got = sign(
            &date,
            method,
            url,
            key,
            secret,
            Some(token),
            region,
            expires_seconds,
            iter::empty(),
            iter::empty(),
        );

        assert_eq!(expected, got.as_str());
    }

    #[test]
    fn aws_headers_example() {
        // Fri, 24 May 2013 00:00:00 GMT
        let date = OffsetDateTime::from_unix_timestamp(1369353600).unwrap();

        let method = Method::Get;
        let url = "https://examplebucket.s3.amazonaws.com/test.txt"
            .parse()
            .unwrap();
        let key = "AKIAIOSFODNN7EXAMPLE";
        let secret = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        let region = "us-east-1";
        let expires_seconds = 86400;

        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=content-type%3Bhost%3Bx-amz-date&X-Amz-Signature=e965ee011ab5dbe8aa2c04a1ff2db8503c0cc117f62ea9274415c0f593ea199f";

        let headers = [
            (
                "content-type",
                "application/x-www-form-urlencoded; charset=utf-8",
            ),
            ("x-amz-date", "20150830T123600Z"),
        ];

        let got = sign(
            &date,
            method,
            url,
            key,
            secret,
            None,
            region,
            expires_seconds,
            iter::empty(),
            headers.iter().copied(),
        );

        assert_eq!(expected, got.as_str());
    }
}
