use std::borrow::Cow;

use time::PrimitiveDateTime;
use url::Url;

use super::util::percent_encode;

const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";

pub fn canonical_request<'a, Q, H, S>(
    method: &str,
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
    string.push_str(method);
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
