use std::fmt::{self, Display};
use std::time::Duration;

use url::Url;

pub use self::get_object::GetObject;
pub use self::multipart_upload::abort::AbortMultipartUpload;
pub use self::multipart_upload::complete::CompleteMultipartUpload;
pub use self::multipart_upload::create::CreateMultipartUpload;
pub use self::multipart_upload::upload::UploadPart;
pub use self::put_object::PutObject;

mod get_object;
mod multipart_upload;
mod put_object;

/// A request which can be signed
pub trait S3Action {
    const METHOD: Method;

    fn sign(&self, expires_at: Duration) -> Url;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Method {
    Head,
    Get,
    Post,
    Put,
    Delete,
}

impl Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Head => "HEAD",
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
        })
    }
}
