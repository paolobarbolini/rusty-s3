use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use url::Url;

// https://perishablepress.com/stop-using-unsafe-characters-in-urls/
pub const FRAGMENT: &AsciiSet = &CONTROLS
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

pub const FRAGMENT_SLASH: &AsciiSet = &FRAGMENT.add(b'/');

pub fn percent_encode(val: &str) -> String {
    utf8_percent_encode(val, FRAGMENT_SLASH).to_string()
}

pub fn percent_encode_path(val: &str) -> String {
    utf8_percent_encode(val, FRAGMENT).to_string()
}

pub fn add_query_params<'a, Q>(mut url: Url, params: Q) -> Url
where
    Q: Iterator<Item = (&'a str, &'a str)>,
{
    let mut query_pairs = url.query_pairs_mut();
    query_pairs.extend_pairs(params);
    drop(query_pairs);

    url
}
