use std::time::Duration;

use rusty_s3::actions::{ListObjectsV2, S3Action};

mod common;

#[tokio::test]
async fn test1() {
    let (bucket, credentials, client) = common::bucket().await;

    let action = bucket.list_objects_v2(Some(&credentials));
    let url = action.sign(Duration::from_secs(60));
    let resp = client
        .get(url)
        .send()
        .await
        .expect("send ListObjectsV2")
        .error_for_status()
        .expect("ListObjectsV2 unexpected status code");
    let text = resp.text().await.expect("ListObjectsV2 read respose body");
    let list = ListObjectsV2::parse_response(&text).expect("ListObjectsV2 parse response");

    assert!(list.contents.is_empty());

    assert_eq!(list.max_keys, 4500);
    assert!(list.common_prefixes.is_empty());
    assert!(list.next_continuation_token.is_none());
    assert!(list.start_after.is_none());

    let body = vec![b'r'; 1024];

    let action = bucket.put_object(Some(&credentials), "test.txt");
    let url = action.sign(Duration::from_secs(60));
    client
        .put(url)
        .body(body.clone())
        .send()
        .await
        .expect("send PutObject")
        .error_for_status()
        .expect("PutObject unexpected status code");

    let action = bucket.head_object(Some(&credentials), "test.txt");
    let url = action.sign(Duration::from_secs(60));

    let resp = client
        .head(url)
        .send()
        .await
        .expect("send HeadObject")
        .error_for_status()
        .expect("HeadObject unexpected status code");

    let content_length = resp
        .headers()
        .get("content-length")
        .expect("Content-Length header")
        .to_str()
        .expect("Content-Length to_str()");
    assert_eq!(content_length, "1024");

    let action = bucket.get_object(Some(&credentials), "test.txt");
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

#[tokio::test]
async fn test_headers() {
    let (bucket, credentials, client) = common::bucket().await;

    let action = bucket.list_objects_v2(Some(&credentials));
    let url = action.sign(Duration::from_secs(60));
    let resp = client
        .get(url)
        .send()
        .await
        .expect("send ListObjectsV2")
        .error_for_status()
        .expect("ListObjectsV2 unexpected status code");
    let text = resp.text().await.expect("ListObjectsV2 read respose body");
    let list = ListObjectsV2::parse_response(&text).expect("ListObjectsV2 parse response");

    assert!(list.contents.is_empty());

    assert_eq!(list.max_keys, 4500);
    assert!(list.common_prefixes.is_empty());
    assert!(list.next_continuation_token.is_none());
    assert!(list.start_after.is_none());

    let body = vec![b'r'; 1024];

    let mut action = bucket.put_object(Some(&credentials), "test.txt");
    action.headers_mut().insert("content-type", "animal/duck");
    let url = action.sign(Duration::from_secs(60));
    client
        .put(url)
        .header("content-type", "animal/duck")
        .body(body.clone())
        .send()
        .await
        .expect("send PutObject")
        .error_for_status()
        .expect("PutObject unexpected status code");

    let action = bucket.get_object(Some(&credentials), "test.txt");
    let url = action.sign(Duration::from_secs(60));

    let resp = client
        .get(url)
        .send()
        .await
        .expect("send GetObject")
        .error_for_status()
        .expect("GetObject unexpected status code");

    assert_eq!(
        resp.headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap(),
        "animal/duck"
    );

    let bytes = resp.bytes().await.expect("GetObject read response body");

    assert_eq!(body, bytes);
}
