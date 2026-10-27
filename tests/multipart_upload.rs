use std::iter;
use std::time::Duration;

use reqwest::StatusCode;
use reqwest::header::ETAG;
use rusty_s3::actions::{CompleteMultipartUploadResponse, CreateMultipartUpload, S3Action as _};

mod common;

#[tokio::test]
async fn complete_multipart_upload() {
    let (bucket, credentials, client) = common::bucket().await;

    let key = "multipart.txt";
    let action = bucket.create_multipart_upload(Some(&credentials), key);
    let url = action.sign(Duration::from_secs(60));
    let text = client
        .post(url)
        .send()
        .await
        .expect("send CreateMultipartUpload")
        .error_for_status()
        .expect("CreateMultipartUpload unexpected status code")
        .text()
        .await
        .expect("CreateMultipartUpload read response body");
    let upload =
        CreateMultipartUpload::parse_response(&text).expect("CreateMultipartUpload parse response");
    let upload_id = upload.upload_id();

    let body = b"123456789".to_vec();
    let action = bucket.upload_part(Some(&credentials), key, 1, upload_id);
    let url = action.sign(Duration::from_secs(60));
    let resp = client
        .put(url)
        .body(body.clone())
        .send()
        .await
        .expect("send UploadPart")
        .error_for_status()
        .expect("UploadPart unexpected status code");
    let etag = resp.headers()[ETAG].to_str().expect("ETag to_str()");

    let action =
        bucket.complete_multipart_upload(Some(&credentials), key, upload_id, iter::once(etag));
    let url = action.sign(Duration::from_secs(60));
    let text = client
        .post(url)
        .body(action.body())
        .send()
        .await
        .expect("send CompleteMultipartUpload")
        .error_for_status()
        .expect("CompleteMultipartUpload unexpected status code")
        .text()
        .await
        .expect("CompleteMultipartUpload read response body");
    // S3 can respond with `200 OK` and still fail, reporting the `Error` in the body
    CompleteMultipartUploadResponse::parse(&text).expect("CompleteMultipartUpload parse response");

    let action = bucket.get_object(Some(&credentials), key);
    let url = action.sign(Duration::from_secs(60));
    let bytes = client
        .get(url)
        .send()
        .await
        .expect("send GetObject")
        .error_for_status()
        .expect("GetObject unexpected status code")
        .bytes()
        .await
        .expect("GetObject read response body");
    assert_eq!(body, bytes);
}

#[tokio::test]
async fn abort_multipart_upload() {
    let (bucket, credentials, client) = common::bucket().await;

    let key = "multipart.txt";
    let action = bucket.create_multipart_upload(Some(&credentials), key);
    let url = action.sign(Duration::from_secs(60));
    let text = client
        .post(url)
        .send()
        .await
        .expect("send CreateMultipartUpload")
        .error_for_status()
        .expect("CreateMultipartUpload unexpected status code")
        .text()
        .await
        .expect("CreateMultipartUpload read response body");
    let upload =
        CreateMultipartUpload::parse_response(&text).expect("CreateMultipartUpload parse response");
    let upload_id = upload.upload_id();

    let action = bucket.upload_part(Some(&credentials), key, 1, upload_id);
    let url = action.sign(Duration::from_secs(60));
    client
        .put(url)
        .body("123456789")
        .send()
        .await
        .expect("send UploadPart")
        .error_for_status()
        .expect("UploadPart unexpected status code");

    let action = bucket.abort_multipart_upload(Some(&credentials), key, upload_id);
    let url = action.sign(Duration::from_secs(60));
    client
        .delete(url)
        .send()
        .await
        .expect("send AbortMultipartUpload")
        .error_for_status()
        .expect("AbortMultipartUpload unexpected status code");

    let action = bucket.list_parts(Some(&credentials), key, upload_id);
    let url = action.sign(Duration::from_secs(60));
    let resp = client.get(url).send().await.expect("send ListParts");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
