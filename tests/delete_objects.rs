use std::time::Duration;

use reqwest::Client;
use url::Url;

use rusty_s3::actions::{ListObjectsV2, ListObjectsV2Response, ObjectIdentifier, S3Action};

mod common;

#[tokio::test]
async fn delete_objects() {
    let (bucket, credentials, client) = common::bucket().await;

    let action = bucket.list_objects_v2(Some(&credentials));
    let list_url = action.sign(Duration::from_secs(600));
    let list = get_objects_list(&client, list_url.clone()).await;
    assert!(list.contents.is_empty());

    // Fill bucket by objects
    let body = vec![b'r'; 1024];
    let mut objects = vec![];
    for i in 0..100 {
        let key = format!("obj{}.txt", i);
        let action = bucket.put_object(Some(&credentials), &key);
        let url = action.sign(Duration::from_secs(60));
        client
            .put(url)
            .body(body.clone())
            .send()
            .await
            .expect("send PutObject")
            .error_for_status()
            .expect("PutObject unexpected status code");
        objects.push(ObjectIdentifier::new(key));
    }

    let list = get_objects_list(&client, list_url.clone()).await;
    assert_eq!(list.contents.len(), 100);

    let action = bucket.delete_objects(Some(&credentials), objects.iter());
    let url = action.sign(Duration::from_secs(60));
    let (body, content_md5) = action.body_with_md5();
    client
        .post(url)
        .header("Content-MD5", content_md5)
        .body(body)
        .send()
        .await
        .expect("send DeleteObjects")
        .error_for_status()
        .expect("DeleteObjects unexpected status code");

    let list = get_objects_list(&client, list_url.clone()).await;
    assert!(list.contents.is_empty());
}

async fn get_objects_list(client: &Client, url: Url) -> ListObjectsV2Response {
    let resp = client
        .get(url)
        .send()
        .await
        .expect("send ListObjectsV2")
        .error_for_status()
        .expect("ListObjectsV2 unexpected status code");
    let text = resp.text().await.expect("ListObjectsV2 read response body");
    ListObjectsV2::parse_response(&text).expect("ListObjectsV2 parse response")
}
