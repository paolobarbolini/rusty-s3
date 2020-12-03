use std::time::Duration;

use url::Url;

pub use self::get_object::GetObject;
pub use self::put_object::PutObject;

mod get_object;
mod put_object;

pub trait S3Action {
    fn sign(&self, expires_at: Duration) -> Url;
}