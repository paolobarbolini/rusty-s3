use std::time::Duration;

use reqwest::Client;
use url::Url;

use rusty_s3::actions::{ListObjectsV2, ListObjectsV2Response, S3Action as _};

mod common;

#[tokio::test]
async fn list_objects_special_keys() {
    let (bucket, credentials, client) = common::bucket().await;

    // Sorted in the UTF-8 binary order S3 lists keys in
    let keys = [
        "100%.txt",
        "a b.txt",
        "a%20b.txt",
        "a+b.txt",
        "dir name/x+y z.txt",
        "ünï cödé.txt",
    ];
    for key in keys {
        let action = bucket.put_object(Some(&credentials), key);
        let url = action.sign(Duration::from_secs(60));
        client
            .put(url)
            .body(key.as_bytes().to_vec())
            .send()
            .await
            .expect("send PutObject")
            .error_for_status()
            .expect("PutObject unexpected status code");
    }

    let action = bucket.list_objects_v2(Some(&credentials));
    let list = list_objects(&client, action.sign(Duration::from_secs(60))).await;
    let listed = list
        .contents
        .iter()
        .map(|content| content.key.as_str())
        .collect::<Vec<_>>();
    assert_eq!(listed, keys);

    let mut action = bucket.list_objects_v2(Some(&credentials));
    action.with_delimiter("/");
    action.with_start_after("a b.txt");
    let list = list_objects(&client, action.sign(Duration::from_secs(60))).await;
    let prefixes = list
        .common_prefixes
        .iter()
        .map(|prefix| prefix.prefix.as_str())
        .collect::<Vec<_>>();
    assert_eq!(prefixes, ["dir name/"]);
    assert_eq!(list.start_after.as_deref(), Some("a b.txt"));
}

async fn list_objects(client: &Client, url: Url) -> ListObjectsV2Response {
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
