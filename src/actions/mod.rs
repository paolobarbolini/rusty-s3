//! S3 request building and response parsing support

use std::time::Duration;

use jiff::Timestamp;
use url::Url;

pub use self::create_bucket::CreateBucket;
pub use self::delete_bucket::DeleteBucket;
pub use self::delete_object::DeleteObject;
#[cfg(feature = "full")]
pub use self::delete_objects::{
    DeleteObjects, DeleteObjectsResponse, DeletedObject, ErrorObject, ObjectIdentifier,
};
#[cfg(feature = "full")]
pub use self::get_bucket_policy::{GetBucketPolicy, GetBucketPolicyResponse};
pub use self::get_object::GetObject;
pub use self::head_bucket::HeadBucket;
pub use self::head_object::HeadObject;
#[cfg(feature = "xml")]
#[doc(inline)]
pub use self::list_objects_v2::{ListObjectsV2, ListObjectsV2Response};
pub use self::multipart_upload::abort::AbortMultipartUpload;
#[cfg(feature = "xml")]
pub use self::multipart_upload::complete::{
    CompleteMultipartUpload, CompleteMultipartUploadResponse,
};
#[cfg(feature = "xml")]
pub use self::multipart_upload::create::{CreateMultipartUpload, CreateMultipartUploadResponse};
#[cfg(feature = "xml")]
pub use self::multipart_upload::list_parts::{ListParts, ListPartsResponse, PartsContent};
pub use self::multipart_upload::upload::UploadPart;
pub use self::put_object::PutObject;
use crate::{Map, Method};

#[cfg(feature = "xml")]
pub(crate) const S3_XML_NS: &str = "http://s3.amazonaws.com/doc/2006-03-01/";

mod create_bucket;
mod delete_bucket;
mod delete_object;
#[cfg(feature = "full")]
mod delete_objects;
#[cfg(feature = "full")]
mod get_bucket_policy;
mod get_object;
mod head_bucket;
mod head_object;
#[cfg(feature = "xml")]
pub mod list_objects_v2;
mod multipart_upload;
mod put_object;

/// A request which can be signed
pub trait S3Action<'a> {
    const METHOD: Method;

    /// Sign a request for this action, using `METHOD` for the [`Method`]
    ///
    /// The returned URL is valid for `expires_in`, starting from now, with a
    /// precision of seconds, so with less than a second it has already
    /// expired. S3 only accepts durations between 1 second and 7 days, and
    /// URLs signed with temporary credentials also stop working when the
    /// credentials expire.
    ///
    /// If the action has no credentials, the URL isn't signed, so
    /// `expires_in` is ignored.
    fn sign(&self, expires_in: Duration) -> Url {
        let now = Timestamp::now();
        self.sign_with_time(expires_in, &now)
    }

    /// Get a mutable reference to the query string of this action
    ///
    /// The `X-Amz-*` query parameters of the signature are added when signing,
    /// so they must not be specified here. See [`sign`](crate::signing::sign)
    /// for the names which every server signs the same way.
    fn query_mut(&mut self) -> &mut Map<'a>;

    /// Get a mutable reference to the signed headers of this action
    ///
    /// Headers specified here must also be present in the final request,
    /// with the same value specified, otherwise the S3 API will return an error.
    /// The `host` header is always signed: unless it's specified here, it's
    /// taken from the URL.
    ///
    /// Conversely, every `x-amz-*` header of the final request, like
    /// `x-amz-acl`, `x-amz-storage-class` or `x-amz-meta-*`, must be specified
    /// here too: AWS rejects requests with `x-amz-*` headers which aren't
    /// signed, even though some S3 compatible servers accept them.
    ///
    /// ```
    /// # use std::time::Duration;
    /// # use rusty_s3::{Bucket, Credentials, S3Action as _, UrlStyle};
    /// # let endpoint = "https://s3.amazonaws.com".parse().unwrap();
    /// # let bucket = Bucket::new(endpoint, UrlStyle::VirtualHost, "rusty-s3", "us-east-1").unwrap();
    /// # let credentials = Credentials::new("key", "secret");
    /// let mut action = bucket.put_object(Some(&credentials), "duck.jpg");
    /// action.headers_mut().insert("x-amz-storage-class", "STANDARD_IA");
    /// let url = action.sign(Duration::from_secs(60 * 60));
    ///
    /// // the `PUT` request to `url` must have the `x-amz-storage-class: STANDARD_IA` header
    /// assert!(url.as_str().contains("X-Amz-SignedHeaders=host%3Bx-amz-storage-class"));
    /// ```
    fn headers_mut(&mut self) -> &mut Map<'a>;

    /// Sign a request for this action, as if it was signed at `time`
    ///
    /// See [`S3Action::sign`]. This is mostly useful for testing.
    fn sign_with_time(&self, expires_in: Duration, time: &Timestamp) -> Url;
}
