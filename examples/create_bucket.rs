use std::error::Error as StdError;
use std::time::Duration;

use reqwest::Client;
use rusty_s3::actions::{CreateBucket, S3Action};
use rusty_s3::{Bucket, Credentials};

const ONE_HOUR: Duration = Duration::from_secs(3600);

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let client = Client::new();

    let url = "http://localhost:9000".parse().unwrap();
    let key = "minioadmin";
    let secret = "minioadmin";
    let region = "minio";

    let bucket = Bucket::new(url, true, "test123".into(), region.into()).unwrap();
    let credential = Credentials::new(key.into(), secret.into());

    let get_obj = CreateBucket::new(&bucket, Some(&credential));
    let url_generated = get_obj.sign(ONE_HOUR);

    client.put(url_generated).send().await?.error_for_status()?;

    Ok(())
}
