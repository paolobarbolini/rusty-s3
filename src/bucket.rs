use std::error::Error as StdError;
use std::fmt::{self, Display};

use url::{ParseError, Url};

use crate::actions::{
    AbortMultipartUpload, CompleteMultipartUpload, CreateBucket, CreateMultipartUpload,
    DeleteBucket, DeleteObject, DeleteObjects, GetObject, HeadObject, ListObjectsV2, ListParts,
    PutObject, UploadPart,
};
use crate::signing::util::percent_encode_path;
use crate::Credentials;

/// An S3 bucket
///
/// ## Path style url
///
/// ```rust
/// # use rusty_s3::{Bucket, UrlStyle};
/// let endpoint = "https://s3.dualstack.eu-west-1.amazonaws.com".parse().expect("endpoint is a valid Url");
/// let path_style = UrlStyle::Path;
/// let name = String::from("rusty-s3");
/// let region = String::from("eu-west-1");
///
/// let bucket = Bucket::new(endpoint, path_style, name, region).expect("Url has a valid scheme and host");
/// assert_eq!(bucket.base_url().as_str(), "https://s3.dualstack.eu-west-1.amazonaws.com/rusty-s3/");
/// assert_eq!(bucket.name(), "rusty-s3");
/// assert_eq!(bucket.region(), "eu-west-1");
/// assert_eq!(bucket.object_url("duck.jpg").expect("url is valid").as_str(), "https://s3.dualstack.eu-west-1.amazonaws.com/rusty-s3/duck.jpg");
/// ```
///
/// ## Domain style url
///
/// ```rust
/// # use rusty_s3::{Bucket, UrlStyle};
/// let endpoint = "https://s3.dualstack.eu-west-1.amazonaws.com".parse().expect("endpoint is a valid Url");
/// let path_style = UrlStyle::VirtualHost;
/// let name = String::from("rusty-s3");
/// let region = String::from("eu-west-1");
///
/// let bucket = Bucket::new(endpoint, path_style, name, region).expect("Url has a valid scheme and host");
/// assert_eq!(bucket.base_url().as_str(), "https://rusty-s3.s3.dualstack.eu-west-1.amazonaws.com/");
/// assert_eq!(bucket.name(), "rusty-s3");
/// assert_eq!(bucket.region(), "eu-west-1");
/// assert_eq!(bucket.object_url("duck.jpg").expect("url is valid").as_str(), "https://rusty-s3.s3.dualstack.eu-west-1.amazonaws.com/duck.jpg");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Bucket {
    base_url: Url,
    name: String,
    region: String,
}

/// The request url format of a S3 bucket.
#[derive(Debug, Clone, Copy)]
pub enum UrlStyle {
    /// requests use the following format
    /// `https://s3.Region.amazonaws.com/bucket-name/key_name`.
    ///
    /// Path style requests are strongly not raccomended,
    /// AWS is plannnig to deprecate them, see [Virtual hosting of buckets](https://docs.aws.amazon.com/AmazonS3/latest/userguide/VirtualHosting.html#virtual-hosted-style-access) for more information.
    Path,
    /// requests use the following format
    /// `https://bucket-name.s3.Region.amazonaws.com/key_name`.
    VirtualHost,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BucketError {
    UnsupportedScheme,
    MissingHost,
}

impl Bucket {
    /// Construct a new S3 bucket
    pub fn new<S: Into<String>>(
        endpoint: Url,
        path_style: UrlStyle,
        name: S,
        region: S,
    ) -> Result<Self, BucketError> {
        endpoint.host_str().ok_or(BucketError::MissingHost)?;

        match endpoint.scheme() {
            "http" | "https" => {}
            _ => return Err(BucketError::UnsupportedScheme),
        };

        let name: String = name.into();
        let base_url = base_url(endpoint, &name, path_style);

        Ok(Self {
            base_url,
            name,
            region: region.into(),
        })
    }

    /// Get the base url of this s3 `Bucket`
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Get the name of this `Bucket`
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the region of this `Bucket`
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Generate an url to an object of this `Bucket`
    ///
    /// This is not a signed url, it's just the starting point for
    /// generating an url to an S3 object.
    pub fn object_url(&self, object: &str) -> Result<Url, ParseError> {
        let object = percent_encode_path(object);
        self.base_url.join(&object)
    }
}

fn base_url(mut endpoint: Url, name: &str, path_style: UrlStyle) -> Url {
    match path_style {
        UrlStyle::Path => {
            let path = format!("{}/", name);
            endpoint.join(&path).unwrap()
        }
        UrlStyle::VirtualHost => {
            let host = format!("{}.{}", name, endpoint.host_str().unwrap());
            endpoint.set_host(Some(&host)).unwrap();
            endpoint
        }
    }
}

// === Bucket level actions ===

impl Bucket {
    /// Create a new bucket.
    ///
    /// See [`CreateBucket`] for more details.
    pub fn create_bucket<'a>(&'a self, credentials: &'a Credentials) -> CreateBucket<'a> {
        CreateBucket::new(self, credentials)
    }

    /// Delete a bucket.
    ///
    /// See [`DeleteBucket`] for more details.
    pub fn delete_bucket<'a>(&'a self, credentials: &'a Credentials) -> DeleteBucket<'a> {
        DeleteBucket::new(self, credentials)
    }
}

// === Basic actions ===

impl Bucket {
    /// Retrieve an object's metadata from S3, using a `HEAD` request.
    ///
    /// See [`HeadObject`] for more details.
    pub fn head_object<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
        object: &'a str,
    ) -> HeadObject<'a> {
        HeadObject::new(self, credentials, object)
    }

    /// Retrieve an object from S3, using a `GET` request.
    ///
    /// See [`GetObject`] for more details.
    pub fn get_object<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
        object: &'a str,
    ) -> GetObject<'a> {
        GetObject::new(self, credentials, object)
    }

    /// List all objects in the bucket.
    ///
    /// See [`ListObjectsV2`] for more details.
    pub fn list_objects_v2<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
    ) -> ListObjectsV2<'a> {
        ListObjectsV2::new(self, credentials)
    }

    /// Upload a file to S3, using a `PUT` request.
    ///
    /// See [`PutObject`] for more details.
    pub fn put_object<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
        object: &'a str,
    ) -> PutObject<'a> {
        PutObject::new(self, credentials, object)
    }

    /// Delete an object from S3, using a `DELETE` request.
    ///
    /// See [`DeleteObject`] for more details.
    pub fn delete_object<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
        object: &'a str,
    ) -> DeleteObject<'a> {
        DeleteObject::new(self, credentials, object)
    }

    /// Delete multiple objects from S3 using a single `POST` request.
    ///
    /// See [`DeleteObjects`] for more details.
    pub fn delete_objects<'a, I>(
        &'a self,
        credentials: Option<&'a Credentials>,
        objects: I,
    ) -> DeleteObjects<'a, I> {
        DeleteObjects::new(self, credentials, objects)
    }
}

// === Multipart Upload ===

impl Bucket {
    /// Create a multipart upload.
    ///
    /// See [`CreateMultipartUpload`] for more details.
    pub fn create_multipart_upload<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
        object: &'a str,
    ) -> CreateMultipartUpload<'a> {
        CreateMultipartUpload::new(self, credentials, object)
    }

    /// Upload a part to a previously created multipart upload.
    ///
    /// See [`UploadPart`] for more details.
    pub fn upload_part<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
        object: &'a str,
        part_number: u16,
        upload_id: &'a str,
    ) -> UploadPart<'a> {
        UploadPart::new(self, credentials, object, part_number, upload_id)
    }

    /// Complete a multipart upload.
    ///
    /// See [`CompleteMultipartUpload`] for more details.
    pub fn complete_multipart_upload<'a, I>(
        &'a self,
        credentials: Option<&'a Credentials>,
        object: &'a str,
        upload_id: &'a str,
        etags: I,
    ) -> CompleteMultipartUpload<'a, I> {
        CompleteMultipartUpload::new(self, credentials, object, upload_id, etags)
    }

    /// Abort multipart upload.
    ///
    /// See [`AbortMultipartUpload`] for more details.
    pub fn abort_multipart_upload<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
        object: &'a str,
        upload_id: &'a str,
    ) -> AbortMultipartUpload<'a> {
        AbortMultipartUpload::new(self, credentials, object, upload_id)
    }

    /// Lists the parts that have been uploaded for a specific multipart upload.
    ///
    /// See [`ListParts`] for more details.
    pub fn list_parts<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
        object: &'a str,
        upload_id: &'a str,
    ) -> ListParts<'a> {
        ListParts::new(self, credentials, object, upload_id)
    }
}

impl Display for BucketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedScheme => f.write_str("unsupported Url scheme"),
            Self::MissingHost => f.write_str("Url is missing the `host`"),
        }
    }
}

impl StdError for BucketError {}

#[cfg(test)]
mod tests {
    use crate::actions::ObjectIdentifier;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn new_pathstyle() {
        let endpoint: Url = "https://s3.dualstack.eu-west-1.amazonaws.com"
            .parse()
            .unwrap();
        let base_url: Url = "https://s3.dualstack.eu-west-1.amazonaws.com/rusty-s3/"
            .parse()
            .unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        let bucket = Bucket::new(endpoint, UrlStyle::Path, name, region).unwrap();

        assert_eq!(bucket.base_url(), &base_url);
        assert_eq!(bucket.name(), name);
        assert_eq!(bucket.region(), region);
    }

    #[test]
    fn new_domainstyle() {
        let endpoint: Url = "https://s3.dualstack.eu-west-1.amazonaws.com"
            .parse()
            .unwrap();
        let base_url: Url = "https://rusty-s3.s3.dualstack.eu-west-1.amazonaws.com"
            .parse()
            .unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        let bucket = Bucket::new(endpoint, UrlStyle::VirtualHost, name, region).unwrap();

        assert_eq!(bucket.base_url(), &base_url);
        assert_eq!(bucket.name(), name);
        assert_eq!(bucket.region(), region);
    }

    #[test]
    fn new_bad_scheme() {
        let endpoint = "ftp://example.com/example".parse().unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        assert_eq!(
            Bucket::new(endpoint, UrlStyle::Path, name, region),
            Err(BucketError::UnsupportedScheme)
        );
    }

    #[test]
    fn new_missing_host() {
        let endpoint = "file:///home/something".parse().unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        assert_eq!(
            Bucket::new(endpoint, UrlStyle::Path, name, region),
            Err(BucketError::MissingHost)
        );
    }

    #[test]
    fn object_url_pathstyle() {
        let endpoint: Url = "https://s3.dualstack.eu-west-1.amazonaws.com"
            .parse()
            .unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        let bucket = Bucket::new(endpoint, UrlStyle::Path, name, region).unwrap();

        let path_style = bucket.object_url("something/cat.jpg").unwrap();
        assert_eq!(
            "https://s3.dualstack.eu-west-1.amazonaws.com/rusty-s3/something/cat.jpg",
            path_style.as_str()
        );
    }

    #[test]
    fn object_url_domainstyle() {
        let endpoint: Url = "https://s3.dualstack.eu-west-1.amazonaws.com"
            .parse()
            .unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        let bucket = Bucket::new(endpoint, UrlStyle::VirtualHost, name, region).unwrap();

        let domain_style = bucket.object_url("something/cat.jpg").unwrap();
        assert_eq!(
            "https://rusty-s3.s3.dualstack.eu-west-1.amazonaws.com/something/cat.jpg",
            domain_style.as_str()
        );
    }

    #[test]
    fn all_actions() {
        let endpoint: Url = "https://s3.dualstack.eu-west-1.amazonaws.com"
            .parse()
            .unwrap();

        let name = "rusty-s3";
        let region = "eu-west-1";
        let bucket = Bucket::new(endpoint, UrlStyle::Path, name, region).unwrap();

        let credentials = Credentials::new(
            "AKIAIOSFODNN7EXAMPLE",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        );

        let _ = bucket.create_bucket(&credentials);
        let _ = bucket.delete_bucket(&credentials);

        let _ = bucket.head_object(Some(&credentials), "duck.jpg");
        let _ = bucket.get_object(Some(&credentials), "duck.jpg");
        let _ = bucket.list_objects_v2(Some(&credentials));
        let _ = bucket.put_object(Some(&credentials), "duck.jpg");
        let _ = bucket.delete_object(Some(&credentials), "duck.jpg");
        let _ = bucket.delete_objects(Some(&credentials), std::iter::empty::<ObjectIdentifier>());

        let _ = bucket.create_multipart_upload(Some(&credentials), "duck.jpg");
        let _ = bucket.upload_part(Some(&credentials), "duck.jpg", 1, "abcd");
        let _ = bucket.complete_multipart_upload(
            Some(&credentials),
            "duck.jpg",
            "abcd",
            ["1234"].iter().copied(),
        );
        let _ = bucket.abort_multipart_upload(Some(&credentials), "duck.jpg", "abcd");
        let _ = bucket.list_parts(Some(&credentials), "duck.jpg", "abcd");
    }
}
