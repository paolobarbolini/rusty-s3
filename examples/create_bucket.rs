use std::error::Error as StdError;
use std::time::Duration;

use reqwest::Client;
use rusty_s3::actions::{CreateBucket, S3Action};
use rusty_s3::{Bucket, Credentials, UrlStyle};

const ONE_HOUR: Duration = Duration::from_secs(3600);

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let client = Client::new();

    let url = "http://localhost:9000".parse().unwrap();
    let key = "minioadmin";
    let secret = "minioadmin";
    let region = "minio";

    let bucket = Bucket::new(url, UrlStyle::Path, "test1234", region).unwrap();
    let credential = Credentials::new(key, secret);

    let action = CreateBucket::new(&bucket, &credential);
    let signed_url = action.sign(ONE_HOUR);

    client.put(signed_url).send().await?.error_for_status()?;

    Ok(())
}
