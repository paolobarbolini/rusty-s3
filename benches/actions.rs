use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rusty_s3::{Bucket, Credentials, S3Action};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Authenticated GetObject", |b| {
        let url = "https://s3.amazonaws.com".parse().unwrap();
        let key = "AKIAIOSFODNN7EXAMPLE";
        let secret = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        let name = "examplebucket";
        let region = "us-east-1";

        let credentials = Credentials::new(key.into(), secret.into());
        let bucket = Bucket::new(url, true, name.into(), region.into()).unwrap();

        b.iter(|| {
            let object = "text.txt";
            let expires_in = Duration::from_secs(60);

            let mut action = bucket.get_object(Some(black_box(&credentials)), black_box(object));
            action
                .query_mut()
                .insert("response-content-type", "text/plain");
            let url = action.sign(black_box(expires_in));
            let _ = url;
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
