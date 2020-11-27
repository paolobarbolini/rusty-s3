use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
use time::PrimitiveDateTime;

type HmacSha256 = Hmac<Sha256>;

pub fn signature(
    date: &PrimitiveDateTime,
    secret: &str,
    region: &str,
    string_to_sign: &str,
) -> String {
    let yyyymmdd = date.format("%Y%m%d");

    let mut mac = HmacSha256::new_varkey(format!("AWS4{}", secret).as_bytes())
        .expect("HMAC can take keys of any size");
    mac.update(yyyymmdd.as_bytes());
    let date_key = mac.finalize().into_bytes();

    let mut mac = HmacSha256::new_varkey(&date_key).expect("HMAC can take keys of any size");
    mac.update(region.as_bytes());
    let date_region_key = mac.finalize().into_bytes();

    let mut mac = HmacSha256::new_varkey(&date_region_key).expect("HMAC can take keys of any size");
    mac.update(b"s3");
    let date_region_service_key = mac.finalize().into_bytes();

    let mut mac =
        HmacSha256::new_varkey(&date_region_service_key).expect("HMAC can take keys of any size");
    mac.update(b"aws4_request");
    let signing_key = mac.finalize().into_bytes();

    let mut mac = HmacSha256::new_varkey(&signing_key).expect("HMAC can take keys of any size");
    mac.update(string_to_sign.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
