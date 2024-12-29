use reqwest::Client;
use std::time::Duration;

use rusty_s3::actions::{CreateMultipartUpload, ListParts, ListPartsResponse, S3Action as _};

mod common;

#[tokio::test]
async fn list_parts() {
    let (bucket, credentials, client) = common::bucket().await;

    // Create multipart upload
    let key = "list_parts";
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

    // Upload some parts
    let part_size: usize = 5 * 1024 * 1024;
    let body = vec![b'r'; part_size];
    for part_num in 0..3u16 {
        let action = bucket.upload_part(Some(&credentials), key, part_num, upload_id);
        let url = action.sign(Duration::from_secs(60));
        client
            .put(url)
            .body(body.clone())
            .send()
            .await
            .expect("send UploadPart")
            .error_for_status()
            .expect("UploadPart unexpected status code");
    }

    // Get list of parts
    let mut action = bucket.list_parts(Some(&credentials), key, upload_id);
    action.set_max_parts(2);
    let parts = get_list_of_parts(&client, action).await;
    assert_eq!(parts.parts.len(), 2);
    assert_eq!(parts.max_parts, 2);
    assert_eq!(parts.next_part_number_marker, Some(1));
    for part in &parts.parts {
        assert_eq!(part.size, part_size as u64);
        assert_eq!(part.etag, "\"0551556e17bba4b6c9dfbaab9e6f08dd\"");
    }

    let mut action = bucket.list_parts(Some(&credentials), key, upload_id);
    action.set_part_number_marker(parts.next_part_number_marker.unwrap());
    let parts = get_list_of_parts(&client, action).await;
    assert_eq!(parts.parts.len(), 1);
    assert!(parts.next_part_number_marker.is_none());
    for part in &parts.parts {
        assert_eq!(part.size, part_size as u64);
        assert_eq!(part.etag, "\"0551556e17bba4b6c9dfbaab9e6f08dd\"");
    }
}

async fn get_list_of_parts(client: &Client, action: ListParts<'_>) -> ListPartsResponse {
    let url = action.sign(Duration::from_secs(60));
    let text = client
        .get(url)
        .send()
        .await
        .expect("send ListParts")
        .error_for_status()
        .expect("ListParts unexpected status code")
        .text()
        .await
        .expect("ListParts read response body");
    ListParts::parse_response(&text).expect("ListParts parse response")
}
