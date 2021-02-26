use std::time::Duration;

use rusty_s3::actions::S3Action;

mod common;

#[tokio::test]
async fn test2() {
    let (bucket, credentials, client) = common::bucket().await;

    let action = bucket.delete_bucket(&credentials);
    let url = action.sign(Duration::from_secs(60));
    client
        .delete(url)
        .send()
        .await
        .expect("send DeleteBucket")
        .error_for_status()
        .expect("DeleteBucket unexpected status code");
}
