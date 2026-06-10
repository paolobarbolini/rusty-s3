use std::borrow::Cow;
use std::fmt::{self, Display};

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use url::Url;

// https://perishablepress.com/stop-using-unsafe-characters-in-urls/
pub(crate) const FRAGMENT: &AsciiSet = &CONTROLS
    // URL_RESERVED
    .add(b':')
    .add(b'?')
    .add(b'#')
    .add(b'[')
    .add(b']')
    .add(b'@')
    .add(b'!')
    .add(b'$')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    .add(b';')
    .add(b'=')
    // URL_UNSAFE
    .add(b'"')
    .add(b' ')
    .add(b'<')
    .add(b'>')
    .add(b'%')
    .add(b'{')
    .add(b'}')
    .add(b'|')
    .add(b'\\')
    .add(b'^')
    .add(b'`');

pub(crate) const FRAGMENT_SLASH: &AsciiSet = &FRAGMENT.add(b'/');

pub(crate) fn percent_encode(val: &str) -> impl Display + Into<Cow<'_, str>> + '_ {
    utf8_percent_encode(val, FRAGMENT_SLASH)
}

pub(crate) fn percent_encode_path(val: &str) -> impl Display + Into<Cow<'_, str>> + '_ {
    utf8_percent_encode(val, FRAGMENT)
}

/// Write a query string, percent-encoding keys and values per RFC 3986.
///
/// This is the encoding AWS Signature Version 4 expects in the canonical
/// request (e.g. a space becomes `%20`, never `+`), so the same function is
/// used to build both the canonical request and the emitted URL query string,
/// guaranteeing the two match.
pub(crate) fn canonical_query_string<'a, Q>(
    query_string: Q,
    mut out: impl fmt::Write,
) -> fmt::Result
where
    Q: Iterator<Item = (&'a str, &'a str)>,
{
    let mut first = true;
    for (key, val) in query_string {
        if first {
            first = false;
        } else {
            out.write_char('&')?;
        }

        write!(out, "{}={}", percent_encode(key), percent_encode(val))?;
    }

    Ok(())
}

pub(crate) fn add_query_params<'a, Q>(mut url: Url, params: Q) -> Url
where
    Q: Iterator<Item = (&'a str, &'a str)>,
{
    // Encode with RFC 3986 percent-encoding (space -> `%20`, not `+`), the same
    // as the signed path, so S3 interprets parameter values such as `prefix`
    // correctly. `Url::query_pairs_mut` would form-encode spaces as `+`.
    let mut query = String::new();
    canonical_query_string(params, &mut query).expect("String writer panicked");

    if !query.is_empty() {
        match url.query() {
            Some(existing) if !existing.is_empty() => {
                url.set_query(Some(&format!("{existing}&{query}")));
            }
            _ => url.set_query(Some(&query)),
        }
    }

    url
}
