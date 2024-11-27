//! S3 request building and response parsing support

use std::time::Duration;

use url::Url;

pub use self::create_bucket::CreateBucket;
pub use self::delete_bucket::DeleteBucket;
pub use self::delete_object::DeleteObject;
#[cfg(feature = "full")]
pub use self::delete_objects::{DeleteObjects, ObjectIdentifier};
pub use self::get_bucket_policy::{GetBucketPolicy, GetBucketPolicyResponse};
pub use self::get_object::GetObject;
pub use self::head_bucket::HeadBucket;
pub use self::head_object::HeadObject;
#[cfg(feature = "full")]
#[doc(inline)]
pub use self::list_objects_v2::{ListObjectsV2, ListObjectsV2Response};
pub use self::multipart_upload::abort::AbortMultipartUpload;
#[cfg(feature = "full")]
pub use self::multipart_upload::complete::CompleteMultipartUpload;
#[cfg(feature = "full")]
pub use self::multipart_upload::create::{CreateMultipartUpload, CreateMultipartUploadResponse};
#[cfg(feature = "full")]
pub use self::multipart_upload::list_parts::{ListParts, ListPartsResponse};
pub use self::multipart_upload::upload::UploadPart;
pub use self::put_object::PutObject;
use crate::{Map, Method};

mod create_bucket;
mod delete_bucket;
mod delete_object;
#[cfg(feature = "full")]
mod delete_objects;
mod get_bucket_policy;
mod get_object;
mod head_bucket;
mod head_object;
#[cfg(feature = "full")]
pub mod list_objects_v2;
mod multipart_upload;
mod put_object;

use time::OffsetDateTime;

/// A request which can be signed
pub trait S3Action<'a> {
    const METHOD: Method;

    /// Sign a request for this action, using `METHOD` for the [`Method`]
    fn sign(&self, expires_in: Duration) -> Url {
        let now = OffsetDateTime::now_utc();
        self.sign_with_time(expires_in, &now)
    }

    /// Get a mutable reference to the query string of this action
    fn query_mut(&mut self) -> &mut Map<'a>;

    /// Get a mutable reference to the signed headers of this action
    ///
    /// Headers specified here must also be present in the final request,
    /// with the same value specified, otherwise the S3 API will return an error.
    fn headers_mut(&mut self) -> &mut Map<'a>;

    /// Takes the time at which the URL should be signed
    /// Used for testing purposes
    fn sign_with_time(&self, expires_in: Duration, time: &OffsetDateTime) -> Url;
}
