use std::time::Duration;

use rusty_s3::actions::{GetObject, S3Action};
use rusty_s3::{Bucket, Credentials};

const ONE_HOUR: Duration = Duration::from_secs(3600);

fn main() {
    let url = "http://localhost:9000".parse().unwrap();
    let key = "minioadmin";
    let secret = "minioadmin";
    let region = "minio";

    let bucket = Bucket::new(url, true, "test".into(), region.into()).unwrap();
    let credential = Credentials::new(key.into(), secret.into());

    let mut action = GetObject::new(&bucket, Some(&credential), "img.jpg");
    action
        .query_mut()
        .insert("response-cache-control", "no-cache, no-store");
    let signed_url = action.sign(ONE_HOUR);

    println!("url: {}", signed_url);
}
