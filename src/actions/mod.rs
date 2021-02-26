//! S3 request building and response parsing support

use std::time::Duration;

use url::Url;

pub use self::create_bucket::CreateBucket;
pub use self::delete_bucket::DeleteBucket;
pub use self::delete_object::DeleteObject;
pub use self::get_object::GetObject;
#[doc(inline)]
pub use self::list_objects_v2::{ListObjectsV2, ListObjectsV2Response};
pub use self::multipart_upload::abort::AbortMultipartUpload;
pub use self::multipart_upload::complete::CompleteMultipartUpload;
pub use self::multipart_upload::create::CreateMultipartUpload;
pub use self::multipart_upload::upload::UploadPart;
pub use self::put_object::PutObject;
use crate::Method;

mod create_bucket;
mod delete_bucket;
mod delete_object;
mod get_object;
pub mod list_objects_v2;
mod multipart_upload;
mod put_object;

/// A request which can be signed
pub trait S3Action {
    const METHOD: Method;

    /// Sign a request for this action, using `METHOD` for the [`Method`]
    fn sign(&self, expires_in: Duration) -> Url;
}
