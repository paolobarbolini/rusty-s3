use jiff::Timestamp;
use zeroize::Zeroizing;

use crate::crypto::hmac_sha256;
use crate::hex::LowerHexWrapper;
use crate::time::YYYYMMDD;

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

    let date_key = hmac_sha256(raw_date.as_bytes(), yyyymmdd.as_bytes());
    let date_region_key = hmac_sha256(&date_key, region.as_bytes());
    let date_region_service_key = hmac_sha256(&date_region_key, b"s3");
    let signing_key = hmac_sha256(&date_region_service_key, b"aws4_request");
    let signature = hmac_sha256(&signing_key, string_to_sign.as_bytes());
    format!("{:x}", LowerHexWrapper(signature))
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
