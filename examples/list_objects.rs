use std::error::Error as StdError;
use std::time::Duration;

use reqwest::Client;
use rusty_s3::actions::{ListObjectsV2, S3Action};
use rusty_s3::{Bucket, Credentials};

const ONE_HOUR: Duration = Duration::from_secs(3600);

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let client = Client::new();

    let url = "http://localhost:9000".parse().unwrap();
    let key = "minioadmin";
    let secret = "minioadmin";
    let region = "minio";

    let bucket = Bucket::new(url, true, "test".into(), region.into()).unwrap();
    let credential = Credentials::new(key.into(), secret.into());

    let action = ListObjectsV2::new(&bucket, Some(&credential));
    let signed_url = action.sign(ONE_HOUR);

    let resp = client.get(signed_url).send().await?.error_for_status()?;
    let text = resp.text().await?;

    println!("{}", text);

    let parsed = ListObjectsV2::parse_response(&text)?;
    println!("{:#?}", parsed);

    Ok(())
}
