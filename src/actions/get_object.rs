use std::time::Duration;
use time::OffsetDateTime;
use url::Url;

use super::S3Action;
use crate::signing::sign;
use crate::{Bucket, Credentials};

pub struct GetObject<'a> {
    bucket: &'a Bucket,
    credentials: Option<&'a Credentials>,
    object: &'a str,
}

impl<'a> GetObject<'a> {
    pub fn new(bucket: &'a Bucket, credentials: Option<&'a Credentials>, object: &'a str) -> Self {
        Self {
            bucket,
            credentials,
            object,
        }
    }

    fn sign_with_time(&self, expires_at: Duration, time: &OffsetDateTime) -> Url {
        let url = self.bucket.object_url(self.object).unwrap();

        match self.credentials {
            Some(credentials) => {
                let url = sign(
                    time,
                    "GET",
                    &url,
                    credentials.key(),
                    credentials.secret(),
                    self.bucket.region(),
                    expires_at.as_secs(),
                );
                url.parse().unwrap()
            }
            None => url,
        }
    }
}

impl<'a> S3Action for GetObject<'a> {
    fn sign(&self, expires_at: Duration) -> Url {
        let now = OffsetDateTime::now_utc();
        self.sign_with_time(expires_at, &now)
    }
}

#[cfg(test)]
mod tests {
    use time::PrimitiveDateTime;

    use super::*;
    use crate::{Bucket, Credentials};

    #[test]
    fn aws_example() {
        let date = PrimitiveDateTime::parse(
            "Fri, 24 May 2013 00:00:00 GMT",
            "%a, %d %b %Y %-H:%M:%S GMT",
        )
        .unwrap()
        .assume_utc();
        let expires_at = Duration::from_secs(86400);

        let endpoint = "https://s3.amazonaws.com".parse().unwrap();
        let bucket =
            Bucket::new(endpoint, false, "examplebucket".into(), "us-east-1".into()).unwrap();
        let credentials = Credentials::new(
            "AKIAIOSFODNN7EXAMPLE".into(),
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".into(),
        );

        let action = GetObject::new(&bucket, Some(&credentials), "test.txt");

        let url = action.sign_with_time(expires_at, &date);
        let expected = "https://examplebucket.s3.amazonaws.com/test.txt?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host&X-Amz-Signature=aeeed9bbccd4d02ee5c0109b86d86835f995330da4c265957d157751f604d404";

        assert_eq!(expected, url.as_str());
    }
}
