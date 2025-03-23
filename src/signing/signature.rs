use hmac::{Hmac, Mac as _};
use jiff::Timestamp;
use sha2::Sha256;
use zeroize::Zeroizing;

use crate::time::YYYYMMDD;

type HmacSha256 = Hmac<Sha256>;

pub(super) fn signature(
    date: &Timestamp,
    secret: &str,
    region: &str,
    string_to_sign: &str,
) -> String {
    let yyyymmdd = date.strftime(&YYYYMMDD).to_string();

    let mut raw_date = String::with_capacity("AWS4".len() + secret.len());
    raw_date.push_str("AWS4");
    raw_date.push_str(secret);
    let raw_date = Zeroizing::new(raw_date);

    let mut mac =
        HmacSha256::new_from_slice(raw_date.as_bytes()).expect("HMAC can take keys of any size");
    mac.update(yyyymmdd.as_bytes());
    let date_key = mac.finalize().into_bytes();

    let mut mac = HmacSha256::new_from_slice(&date_key).expect("HMAC can take keys of any size");
    mac.update(region.as_bytes());
    let date_region_key = mac.finalize().into_bytes();

    let mut mac =
        HmacSha256::new_from_slice(&date_region_key).expect("HMAC can take keys of any size");
    mac.update(b"s3");
    let date_region_service_key = mac.finalize().into_bytes();

    let mut mac = HmacSha256::new_from_slice(&date_region_service_key)
        .expect("HMAC can take keys of any size");
    mac.update(b"aws4_request");
    let signing_key = mac.finalize().into_bytes();

    let mut mac = HmacSha256::new_from_slice(&signing_key).expect("HMAC can take keys of any size");
    mac.update(string_to_sign.as_bytes());
    format!("{:x}", mac.finalize().into_bytes())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn aws_example() {
        // Fri, 24 May 2013 00:00:00 GMT
        let date = Timestamp::from_second(1369353600).unwrap();

        let secret = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        let region = "us-east-1";

        let expected = "aeeed9bbccd4d02ee5c0109b86d86835f995330da4c265957d157751f604d404";

        let got = signature(&date, secret, region, create_string_to_sign());

        assert_eq!(got, expected);
    }

    fn create_string_to_sign() -> &'static str {
        concat!(
            "AWS4-HMAC-SHA256\n",
            "20130524T000000Z\n",
            "20130524/us-east-1/s3/aws4_request\n",
            "3bfa292879f6447bbcda7001decf97f4a54dc650c8942174ae0a9121cf58ad04"
        )
    }
}
