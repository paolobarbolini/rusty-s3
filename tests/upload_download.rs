use std::time::Duration;

use rusty_s3::actions::{GetObject, PutObject, S3Action};

mod common;

#[tokio::test]
async fn test1() {
    let (bucket, credentials, client) = common::bucket().await;

    let body = vec![b'r'; 1024];

    let action = PutObject::new(&bucket, Some(&credentials), "test.txt");
    let url = action.sign(Duration::from_secs(60));
    client
        .put(url)
        .body(body.clone())
        .send()
        .await
        .expect("send PutObject")
        .error_for_status()
        .expect("PutObject unexpected status code");

    let action = GetObject::new(&bucket, Some(&credentials), "test.txt");
    let url = action.sign(Duration::from_secs(60));

    let resp = client
        .get(url)
        .send()
        .await
        .expect("send GetObject")
        .error_for_status()
        .expect("GetObject unexpected status code");
    let bytes = resp.bytes().await.expect("GetObject read response body");

    assert_eq!(body, bytes);
}
