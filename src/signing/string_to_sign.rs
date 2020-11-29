use sha2::{Digest, Sha256};
use time::OffsetDateTime;

pub fn string_to_sign(date: &OffsetDateTime, region: &str, canonical_request: &str) -> String {
    let iso8601 = date.lazy_format("%Y%m%dT%H%M%SZ");
    let yyyymmdd = date.lazy_format("%Y%m%d");
    let scope = format!("{}/{}/s3/aws4_request", yyyymmdd, region);
    let mut hasher = Sha256::new();
    hasher.update(canonical_request.as_bytes());
    let hash = hasher.finalize();
    let hash = hex::encode(hash);

    format!("AWS4-HMAC-SHA256\n{}\n{}\n{}", iso8601, scope, hash)
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
        .unwrap()
        .assume_utc();

        let region = "us-east-1";

        let expected = concat!(
            "AWS4-HMAC-SHA256\n",
            "20130524T000000Z\n",
            "20130524/us-east-1/s3/aws4_request\n",
            "3bfa292879f6447bbcda7001decf97f4a54dc650c8942174ae0a9121cf58ad04"
        );

        let got = string_to_sign(&date, region, create_canonical_request());

        assert_eq!(got, expected);
    }

    fn create_canonical_request() -> &'static str {
        concat!(
            "GET\n",
            "/test.txt\n",
            "X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host\n",
            "host:examplebucket.s3.amazonaws.com\n",
            "\n",
            "host\n",
            "UNSIGNED-PAYLOAD",
        )
    }
}
