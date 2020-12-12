use std::time::Duration;

use reqwest::Client;
use rusty_s3::actions::{CreateBucket, S3Action};
use rusty_s3::{Bucket, Credentials};

pub async fn bucket() -> (Bucket, Credentials, Client) {
    let mut buf = [0; 8];
    getrandom::getrandom(&mut buf).expect("getrandom");

    let hex = hex::encode(&buf);
    let name = format!("test-{}", hex);

    let url = "http://localhost:9000".parse().unwrap();
    let key = "minioadmin";
    let secret = "minioadmin";
    let region = "minio";

    let bucket = Bucket::new(url, true, name, region.into()).unwrap();
    let credentials = Credentials::new(key.into(), secret.into());

    let client = Client::new();
    let action = CreateBucket::new(&bucket, Some(&credentials));
    let url = action.sign(Duration::from_secs(60));
    client
        .put(url)
        .send()
        .await
        .expect("send CreateBucket request")
        .error_for_status()
        .expect("CreateBucket request unexpected status code");

    (bucket, credentials, client)
}
