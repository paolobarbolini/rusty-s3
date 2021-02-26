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

    assert_eq!(list.contents.is_empty(), true);

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
