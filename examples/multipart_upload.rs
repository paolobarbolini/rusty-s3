use std::error::Error as StdError;
use std::iter;
use std::time::Duration;

use reqwest::header::ETAG;
use reqwest::Client;
use rusty_s3::actions::{CompleteMultipartUpload, CreateMultipartUpload, S3Action, UploadPart};
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

    let action = CreateMultipartUpload::new(&bucket, Some(&credential), "idk.txt");
    let url = action.sign(ONE_HOUR);
    let resp = client.post(url).send().await?.error_for_status()?;
    let body = resp.text().await?;

    let multipart = CreateMultipartUpload::parse_response(&body)?;

    println!(
        "multipart upload created - upload id: {}",
        multipart.upload_id()
    );

    let part_upload = UploadPart::new(
        &bucket,
        Some(&credential),
        "idk.txt",
        1,
        multipart.upload_id(),
    );
    let url = part_upload.sign(ONE_HOUR);

    let body = "123456789";
    let resp = client
        .put(url)
        .body(body)
        .send()
        .await?
        .error_for_status()?;
    let etag = resp
        .headers()
        .get(ETAG)
        .expect("every UploadPart request returns an Etag");

    println!("etag: {}", etag.to_str().unwrap());

    let action = CompleteMultipartUpload::new(
        &bucket,
        Some(&credential),
        "idk.txt",
        multipart.upload_id(),
        iter::once(etag.to_str().unwrap()),
    );
    let url = action.sign(ONE_HOUR);

    let resp = client
        .post(url)
        .body(action.body())
        .send()
        .await?
        .error_for_status()?;
    let body = resp.text().await?;
    println!("it worked! {}", body);

    Ok(())
}
