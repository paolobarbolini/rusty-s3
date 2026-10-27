use std::error::Error as StdError;
use std::iter;
use std::time::Duration;

use reqwest::Client;
use reqwest::header::ETAG;
use rusty_s3::actions::{
    AbortMultipartUpload, CompleteMultipartUpload, CompleteMultipartUploadResponse,
    CreateMultipartUpload, S3Action as _, UploadPart,
};
use rusty_s3::{Bucket, Credentials, UrlStyle};

const ONE_HOUR: Duration = Duration::from_secs(3600);

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let client = Client::new();

    let url = "http://localhost:9000".parse().unwrap();
    let key = "minioadmin";
    let secret = "minioadmin";
    let region = "minio";

    let bucket = Bucket::new(url, UrlStyle::Path, "test", region).unwrap();
    let credential = Credentials::new(key, secret);

    let action = CreateMultipartUpload::new(&bucket, Some(&credential), "idk.txt");
    let url = action.sign(ONE_HOUR);
    let resp = client.post(url).send().await?.error_for_status()?;
    let body = resp.text().await?;

    let multipart = CreateMultipartUpload::parse_response(&body)?;

    println!(
        "multipart upload created - upload id: {}",
        multipart.upload_id()
    );

    if let Err(err) = upload(&client, &bucket, &credential, multipart.upload_id()).await {
        // the uploaded parts consume storage until the upload is completed or
        // aborted, and uploads can still be left behind, like when the
        // process crashes, so the bucket should also have a lifecycle rule
        // aborting incomplete uploads
        let action =
            AbortMultipartUpload::new(&bucket, Some(&credential), "idk.txt", multipart.upload_id());
        let url = action.sign(ONE_HOUR);
        client.delete(url).send().await?.error_for_status()?;

        println!("multipart upload aborted");
        return Err(err);
    }

    Ok(())
}

async fn upload(
    client: &Client,
    bucket: &Bucket,
    credential: &Credentials,
    upload_id: &str,
) -> Result<(), Box<dyn StdError>> {
    let part_upload = UploadPart::new(bucket, Some(credential), "idk.txt", 1, upload_id);
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
        .ok_or("UploadPart response is missing the ETag")?
        .to_str()?;

    println!("etag: {etag}");

    let action = CompleteMultipartUpload::new(
        bucket,
        Some(credential),
        "idk.txt",
        upload_id,
        iter::once(etag),
    );
    let url = action.sign(ONE_HOUR);

    let resp = client
        .post(url)
        .body(action.body())
        .send()
        .await?
        .error_for_status()?;
    let body = resp.text().await?;
    // S3 can respond with `200 OK` and still fail, reporting the `Error` in the body
    let completed = CompleteMultipartUploadResponse::parse(&body)
        .map_err(|err| format!("CompleteMultipartUpload failed: {err}: {body}"))?;
    println!("it worked! etag: {}", completed.etag());

    Ok(())
}
