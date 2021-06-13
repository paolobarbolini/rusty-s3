use std::time::Duration;

use rusty_s3::actions::{GetObject, S3Action};
use rusty_s3::{Bucket, Credentials};

const ONE_HOUR: Duration = Duration::from_secs(3600);

fn main() {
    let url = "http://172.22.98.45:9000".parse().unwrap();
    let key = "ccc";
    let secret = "WXZFwxzf123";
    let region = "minio";

    let bucket = Bucket::new(url, true, "bucket0".into(), region.into()).unwrap();
    let credential = Credentials::new(key.into(), secret.into());

    let mut action = GetObject::new(&bucket, Some(&credential), "test.md");
    action
        .query_mut()
        .insert("response-cache-control", "no-cache, no-store");
    let signed_url = action.sign(ONE_HOUR);

    println!("url: {}", signed_url);
}
