use rusty_s3::sign;
use time::OffsetDateTime;

fn main() {
    let date = OffsetDateTime::now_utc();

    let url = "http://localhost:9000/test/img.jpg".parse().unwrap();
    let key = "minioadmin";
    let secret = "minioadmin";
    let region = "minio";
    let expires_seconds = 3600;

    let signed = sign(&date, "GET", &url, key, secret, region, expires_seconds);
    println!("signed: {}", signed);
}
