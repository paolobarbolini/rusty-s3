use std::time::Duration;

use reqwest::Client;
use rusty_s3::actions::{CreateBucket, S3Action as _};
use rusty_s3::{Bucket, Credentials, UrlStyle};

pub(crate) async fn bucket() -> (Bucket, Credentials, Client) {
    let mut buf = [0; 8];
    getrandom::fill(&mut buf).expect("getrandom");

    let hex = hex::encode(buf);
    let name = format!("test-{hex}");

    let url = "http://localhost:9000".parse().unwrap();
    let key = "minioadmin";
    let secret = "minioadmin";
    let region = "minio";

    let bucket = Bucket::new(url, UrlStyle::Path, name, region).unwrap();
    let credentials = Credentials::new(key, secret);

    let client = Client::new();
    let action = CreateBucket::new(&bucket, &credentials);
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
