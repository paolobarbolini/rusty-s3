use sha2::{Digest, Sha256};
use time::PrimitiveDateTime;

pub fn string_to_sign(date: &PrimitiveDateTime, region: &str, canonical_request: &str) -> String {
    let iso8601 = date.lazy_format("%Y%m%dT%H%M%SZ"); //"2020-11-26T15:16Z";
    let yyyymmdd = date.lazy_format("%Y%m%d");
    let scope = format!("{}/{}/s3/aws4_request", yyyymmdd, region);
    let mut hasher = Sha256::new();
    hasher.update(canonical_request.as_bytes());
    let hash = hasher.finalize();
    let hash = hex::encode(hash);

    format!("AWS4-HMAC-SHA256\n{}\n{}\n{}", iso8601, scope, hash)
}

mod test {
    use super::*;

    #[test]
    fn test1() {}
}
