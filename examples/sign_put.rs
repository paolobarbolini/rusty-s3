use std::time::Duration;

use rusty_s3::actions::{PutObject, S3Action};
use rusty_s3::{Bucket, Credentials};

const ONE_HOUR: Duration = Duration::from_secs(3600);

fn main() {
    let url = "http://localhost:9000".parse().unwrap();
    let key = "minioadmin";
    let secret = "minioadmin";
    let region = "minio";

    let bucket = Bucket::new(url, true, "test".into(), region.into()).unwrap();
    let credential = Credentials::new(key.into(), secret.into());

    let get_obj = PutObject::new(&bucket, Some(&credential), "duck.jpg");
    let url_generated = get_obj.sign(ONE_HOUR);

    println!("url: {}", url_generated);
}
