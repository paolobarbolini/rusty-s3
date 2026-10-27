use std::time::Duration;

use reqwest::Client;
use url::Url;

use rusty_s3::actions::{
    DeleteObjectsResponse, DeletedObject, ErrorObject, ListObjectsV2, ListObjectsV2Response,
    ObjectIdentifier, S3Action as _,
};

mod common;

#[tokio::test]
#[ignore = "versitygw puts the response outside of the S3 namespace until a release with https://github.com/versity/versitygw/pull/2579"]
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
        let key = format!("obj{i}.txt");
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
    let text = client
        .post(url)
        .header("Content-MD5", content_md5)
        .body(body)
        .send()
        .await
        .expect("send DeleteObjects")
        .error_for_status()
        .expect("DeleteObjects unexpected status code")
        .text()
        .await
        .expect("DeleteObjects read response body");
    let response = DeleteObjectsResponse::parse(&text).expect("DeleteObjects parse response");
    let errors: &[ErrorObject] = &response.errors;
    assert!(errors.is_empty(), "{errors:?}");
    let deleted: &[DeletedObject] = &response.deleted;
    assert_eq!(deleted.len(), 100);

    let list = get_objects_list(&client, list_url.clone()).await;
    assert!(list.contents.is_empty());
}

#[tokio::test]
async fn delete_objects_carriage_return() {
    let (bucket, credentials, client) = common::bucket().await;

    // XML parsers turn a raw `\r` into `\n`, which would delete the decoy
    let target = "Icon\r";
    let decoy = "Icon\n";
    for key in [target, decoy] {
        let action = bucket.put_object(Some(&credentials), key);
        let url = action.sign(Duration::from_secs(60));
        client
            .put(url)
            .send()
            .await
            .expect("send PutObject")
            .error_for_status()
            .expect("PutObject unexpected status code");
    }

    let objects = [ObjectIdentifier::new(target.to_owned())];
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

    let action = bucket.list_objects_v2(Some(&credentials));
    let list = get_objects_list(&client, action.sign(Duration::from_secs(60))).await;
    let keys = list
        .contents
        .iter()
        .map(|content| content.key.as_str())
        .collect::<Vec<_>>();
    assert_eq!(keys, [decoy]);
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
