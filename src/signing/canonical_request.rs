use url::Url;

use super::util::percent_encode;
use crate::Method;

const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";

pub fn canonical_request<'a, Q, H, S>(
    method: Method,
    url: &Url,
    query_string: Q,
    headers: H,
    signed_headers: S,
) -> String
where
    Q: Iterator<Item = (&'a str, &'a str)>,
    H: Iterator<Item = (&'a str, &'a str)>,
    S: Iterator<Item = &'a str>,
{
    let mut string = String::with_capacity(64);
    string.push_str(method.to_str());
    string.push('\n');
    string.push_str(url.path());
    string.push('\n');

    canonical_query_string(query_string, &mut string);

    string.push('\n');

    canonical_headers(headers, &mut string);

    string.push('\n');

    signed_headers_(signed_headers, &mut string);

    string.push('\n');

    string.push_str(UNSIGNED_PAYLOAD);

    string
}

fn canonical_query_string<'a, Q>(query_string: Q, string: &mut String)
where
    Q: Iterator<Item = (&'a str, &'a str)>,
{
    let mut first = true;
    for (key, val) in query_string {
        if first {
            first = false;
        } else {
            string.push('&');
        }

        string.push_str(&percent_encode(key));
        string.push('=');
        string.push_str(&percent_encode(val));
    }
}

fn canonical_headers<'a, H>(headers: H, string: &mut String)
where
    H: Iterator<Item = (&'a str, &'a str)>,
{
    for (key, val) in headers {
        string.push_str(key);
        string.push(':');
        string.push_str(val.trim());

        string.push('\n');
    }
}

fn signed_headers_<'a, H>(signed_headers: H, string: &mut String)
where
    H: Iterator<Item = &'a str>,
{
    let mut first = true;
    for key in signed_headers {
        if first {
            first = false;
        } else {
            string.push(';');
        }

        string.push_str(key);
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use time::PrimitiveDateTime;

    use super::*;
    use crate::Method;

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
        let region = "us-east-1";
        let key = "AKIAIOSFODNN7EXAMPLE";
        let expires_seconds = 86400;

        let date_str = date.format("%Y%m%dT%H%M%SZ");
        let yyyymmdd = date.format("%Y%m%d");

        let credential = format!(
            "{}/{}/{}/{}/{}",
            key, yyyymmdd, region, "s3", "aws4_request"
        );
        let signed_headers_str = "host";

        let expected = concat!(
            "GET\n",
            "/test.txt\n",
            "X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host\n",
            "host:examplebucket.s3.amazonaws.com\n",
            "\n",
            "host\n",
            "UNSIGNED-PAYLOAD",
        );

        let got = canonical_request(
            method,
            &url,
            vec![
                ("X-Amz-Algorithm", "AWS4-HMAC-SHA256"),
                ("X-Amz-Credential", &credential),
                ("X-Amz-Date", &date_str),
                ("X-Amz-Expires", &expires_seconds.to_string()),
                ("X-Amz-SignedHeaders", signed_headers_str),
            ]
            .into_iter(),
            vec![("host", url.host_str().unwrap())].into_iter(),
            vec!["host"].into_iter(),
        );

        assert_eq!(got, expected);
    }
}
