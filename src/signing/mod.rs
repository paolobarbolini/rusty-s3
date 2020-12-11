use std::{iter, slice, str};

use time::OffsetDateTime;
use url::Url;

use crate::sorting_iter::SortingIterator;
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
    region: &str,
    expires_seconds: u64,

    query_string: Q,
    headers: H,
) -> Url
where
    Q: Iterator<Item = (&'a str, &'a str)> + Clone,
    H: Iterator<Item = (&'a str, &'a str)> + Clone,
{
    let yyyymmdd = date.format("%Y%m%d");

    let credential = format!(
        "{}/{}/{}/{}/{}",
        key, yyyymmdd, region, "s3", "aws4_request"
    );
    let date_str = date.format("%Y%m%dT%H%M%SZ");
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

    // SAFETY: this is a workaround for the compiler thinking thet host_header, credential,
    // date_str and expires_seconds_string have to live as long as &'a str.
    // These parementers outlive the functions taking them, and we make sure of it by
    // trying to access them before returning. This makes sure we haven't dropped them
    // or moved them to another function.
    let host_header_ = unsafe {
        let s = host_header.as_str();
        str::from_utf8_unchecked(slice::from_raw_parts(s.as_ptr(), s.len()))
    };
    let credential_ = unsafe {
        let s = credential.as_str();
        str::from_utf8_unchecked(slice::from_raw_parts(s.as_ptr(), s.len()))
    };
    let date_str_ = unsafe {
        let s = date_str.as_str();
        str::from_utf8_unchecked(slice::from_raw_parts(s.as_ptr(), s.len()))
    };
    let expires_seconds_string_ = unsafe {
        let s = expires_seconds_string.as_str();
        str::from_utf8_unchecked(slice::from_raw_parts(s.as_ptr(), s.len()))
    };

    let standard_headers = iter::once(("host", host_header_));
    let headers = SortingIterator::new(standard_headers, headers);

    let standard_query = [
        ("X-Amz-Algorithm", "AWS4-HMAC-SHA256"),
        ("X-Amz-Credential", credential_),
        ("X-Amz-Date", date_str_),
        ("X-Amz-Expires", expires_seconds_string_),
        ("X-Amz-SignedHeaders", "host"),
    ];

    let query_string = SortingIterator::new(standard_query.iter().copied(), query_string);

    {
        let mut query_pairs = url.query_pairs_mut();
        query_pairs.clear();

        query_pairs.extend_pairs(query_string.clone());
    }

    let canonical_req = canonical_request::canonical_request(
        method,
        &url,
        query_string,
        headers,
        iter::once("host"),
    );

    let signed_string = string_to_sign::string_to_sign(date, region, &canonical_req);

    let signature = signature::signature(date, secret, region, &signed_string);

    // SAFETY: here to verify the safety of the above unsafe functions, by making sure
    // the borrowed values are still alive.
    let _ = host_header.as_str();
    let _ = credential.as_str();
    let _ = date_str.as_str();
    let _ = expires_seconds_string.as_str();

    url.query_pairs_mut()
        .append_pair("X-Amz-Signature", &signature);
    url
}

#[cfg(test)]
mod tests {
    use std::iter;

    use pretty_assertions::assert_eq;
    use time::PrimitiveDateTime;

    use super::Method;
    use super::*;

    #[test]
    fn aws_example() {
        let date = PrimitiveDateTime::parse(
            "Fri, 24 May 2013 00:00:00 GMT",
            "%a, %d %b %Y %-H:%M:%S GMT",
        )
        .unwrap()
        .assume_utc();
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
            region,
            expires_seconds,
            iter::empty(),
            iter::empty(),
        );

        assert_eq!(expected, got.as_str());
    }
}
