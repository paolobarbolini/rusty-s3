use std::time::Duration;

use url::Url;

pub use self::get_object::GetObject;

mod get_object;

pub trait S3Action {
    fn sign(&self, expires_at: Duration) -> Url;
}
