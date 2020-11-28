use time::PrimitiveDateTime;
use url::Url;

mod canonical_request;
mod signature;
mod string_to_sign;
pub(crate) mod util;

pub fn sign(
    date: &PrimitiveDateTime,
    method: &str,
    url: &Url,
    key: &str,
    secret: &str,
    region: &str,
    expires_seconds: u64,
) -> String {
    let yyyymmdd = date.format("%Y%m%d");

    let credential_ = format!(
        "{}/{}/{}/{}/{}",
        key, yyyymmdd, region, "s3", "aws4_request"
    );
    let credential = util::percent_encode(&credential_);
    let date_str = date.format("%Y%m%dT%H%M%SZ");
    let signed_headers_str = "host";
    let url_query = format!("{}?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential={}&X-Amz-Date={}&X-Amz-Expires={}&X-Amz-SignedHeaders={}",url.to_string(),credential,date_str,expires_seconds,signed_headers_str);

    let host_header = match (url.scheme(), url.port()) {
        ("http", None) | ("http", Some(80)) | ("https", None) | ("https", Some(443)) => {
            url.host_str().unwrap().to_string()
        }
        ("http", Some(port)) | ("https", Some(port)) => {
            format!("{}:{}", url.host_str().unwrap(), port)
        }
        _ => panic!("unsupported url scheme"),
    };

    let canonical_req = canonical_request::canonical_request(
        method,
        &url,
        vec![
            ("X-Amz-Algorithm", "AWS4-HMAC-SHA256"),
            ("X-Amz-Credential", &credential_),
            ("X-Amz-Date", &date_str),
            ("X-Amz-Expires", &expires_seconds.to_string()),
            ("X-Amz-SignedHeaders", signed_headers_str),
        ]
        .into_iter(),
        vec![("host", host_header.as_ref())].into_iter(),
        vec!["host"].into_iter(),
    );

    let signed_string = string_to_sign::string_to_sign(date, region, &canonical_req);

    let signature = signature::signature(date, secret, region, &signed_string);

    format!("{}&X-Amz-Signature={}", url_query, signature)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use time::PrimitiveDateTime;

    use super::*;

    #[test]
    fn aws_example() {
        let date = PrimitiveDateTime::parse(
            "Fri, 24 May 2013 00:00:00 GMT",
            "%a, %d %b %Y %-H:%M:%S GMT",
        )
        .unwrap();
        let method = "GET";
        let url = "https://examplebucket.s3.amazonaws.com/test.txt"
            .parse()
            .unwrap();
        let key = "AKIAIOSFODNN7EXAMPLE";
        let secret = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        let region = "us-east-1";
        let expires_seconds = 86400;

        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&X-Amz-Signature=aeeed9bbccd4d02ee5c0109b86d86835f995330da4c265957d157751f604d404";

        let got = sign(&date, method, &url, key, secret, region, expires_seconds);

        assert_eq!(expected, got);
    }
}
