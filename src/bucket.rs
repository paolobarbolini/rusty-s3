use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt::{self, Display};

use url::{ParseError, Url};

use crate::Credentials;
use crate::actions::{
    AbortMultipartUpload, CreateBucket, DeleteBucket, DeleteObject, GetObject, HeadBucket,
    HeadObject, PutObject, UploadPart,
};
#[cfg(feature = "xml")]
use crate::actions::{CompleteMultipartUpload, CreateMultipartUpload, ListObjectsV2, ListParts};
#[cfg(feature = "full")]
use crate::actions::{DeleteObjects, GetBucketPolicy};
use crate::signing::util::{percent_encode, percent_encode_path};

/// An S3 bucket
///
/// ## Path style url
///
/// ```rust
/// # use rusty_s3::{Bucket, UrlStyle};
/// let endpoint = "https://s3.dualstack.eu-west-1.amazonaws.com".parse().expect("endpoint is a valid Url");
/// let path_style = UrlStyle::Path;
/// let name = "rusty-s3";
/// let region = "eu-west-1";
///
/// let bucket = Bucket::new(endpoint, path_style, name, region).expect("Url has a valid scheme and host");
/// assert_eq!(bucket.base_url().as_str(), "https://s3.dualstack.eu-west-1.amazonaws.com/rusty-s3/");
/// assert_eq!(bucket.name(), "rusty-s3");
/// assert_eq!(bucket.region(), "eu-west-1");
/// assert_eq!(bucket.object_url("duck.jpg").as_str(), "https://s3.dualstack.eu-west-1.amazonaws.com/rusty-s3/duck.jpg");
/// ```
///
/// ## Domain style url
///
/// ```rust
/// # use rusty_s3::{Bucket, UrlStyle};
/// let endpoint = "https://s3.dualstack.eu-west-1.amazonaws.com".parse().expect("endpoint is a valid Url");
/// let path_style = UrlStyle::VirtualHost;
/// let name = "rusty-s3";
/// let region = "eu-west-1";
///
/// let bucket = Bucket::new(endpoint, path_style, name, region).expect("Url has a valid scheme and host");
/// assert_eq!(bucket.base_url().as_str(), "https://rusty-s3.s3.dualstack.eu-west-1.amazonaws.com/");
/// assert_eq!(bucket.name(), "rusty-s3");
/// assert_eq!(bucket.region(), "eu-west-1");
/// assert_eq!(bucket.object_url("duck.jpg").as_str(), "https://rusty-s3.s3.dualstack.eu-west-1.amazonaws.com/duck.jpg");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bucket {
    base_url: Url,
    name: Cow<'static, str>,
    region: Cow<'static, str>,
}

/// The request url format of a S3 bucket.
#[derive(Debug, Clone, Copy)]
pub enum UrlStyle {
    /// Requests will use "path-style" url: i.e:
    /// `https://s3.<region>.amazonaws.com/<bucket>/<key>`.
    ///
    /// This style should be considered deprecated and is **NOT RECOMMENDED**.
    /// Check [Amazon S3 Path Deprecation Plan](https://aws.amazon.com/blogs/aws/amazon-s3-path-deprecation-plan-the-rest-of-the-story/)
    /// for more informations.
    Path,
    /// Requests will use "virtual-hosted-style" urls, i.e:
    /// `https://<bucket>.s3.<region>.amazonaws.com/<key>`.
    VirtualHost,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BucketError {
    UnsupportedScheme,
    MissingHost,
    ParseError(ParseError),
}

impl From<ParseError> for BucketError {
    fn from(error: ParseError) -> Self {
        Self::ParseError(error)
    }
}

impl Bucket {
    /// Construct a new S3 bucket
    ///
    /// The path of the `endpoint`, if any, is kept as a prefix of the urls of
    /// the bucket: for example, with [`UrlStyle::Path`],
    /// `https://example.com/s3` gives `https://example.com/s3/<bucket>/<key>`.
    /// Its query string and fragment are ignored: use
    /// [`S3Action::query_mut`](crate::S3Action::query_mut) to add query
    /// parameters to a request. Its username and password are ignored too,
    /// as they would end up in every presigned url: configure the HTTP client
    /// instead, if a proxy in front of S3 needs them.
    ///
    /// # Errors
    ///
    /// Returns a `BucketError` if the `endpoint` is not a valid url, if the `endpoint` is missing the host,
    /// or if the `name` can't be part of a host name when using [`UrlStyle::VirtualHost`].
    pub fn new(
        endpoint: Url,
        path_style: UrlStyle,
        name: impl Into<Cow<'static, str>>,
        region: impl Into<Cow<'static, str>>,
    ) -> Result<Self, BucketError> {
        endpoint.host_str().ok_or(BucketError::MissingHost)?;

        match endpoint.scheme() {
            "http" | "https" => {}
            _ => return Err(BucketError::UnsupportedScheme),
        }

        let name = name.into();
        let region = region.into();

        let base_url = base_url(endpoint, &name, path_style)?;

        Ok(Self {
            base_url,
            name,
            region,
        })
    }

    /// Get the base url of this s3 `Bucket`
    #[must_use]
    pub const fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Get the name of this `Bucket`
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the region of this `Bucket`
    #[must_use]
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Generate an url to an object of this `Bucket`
    ///
    /// This is not a signed url, it's just the starting point for
    /// generating an url to an S3 object.
    #[must_use]
    pub fn object_url(&self, object: &str) -> Url {
        // Appended to the path of the bucket rather than resolved against
        // it, so that a key starting with `/` can't replace the path of the
        // bucket, nor one starting with `//` its host
        let mut url = self.base_url.clone();
        let path = format!("{}{}", url.path(), percent_encode_path(object));
        url.set_path(&path);
        url
    }
}

fn base_url(mut endpoint: Url, name: &str, path_style: UrlStyle) -> Result<Url, ParseError> {
    // Signing replaces the query string, so the one of the endpoint would
    // only be kept by some unsigned requests
    endpoint.set_query(None);
    endpoint.set_fragment(None);

    // Credentials would end up in every url, presigned ones included, and
    // HTTP clients send them in an `Authorization` header, which S3 rejects
    // next to a presigned query string. Removing them only fails for urls
    // which can't have any, like the ones without a host.
    let _ = endpoint.set_username("");
    let _ = endpoint.set_password(None);

    // The path of the endpoint is a prefix of every url. Without a trailing
    // `/`, the name of the bucket or the key would be appended to its last
    // segment.
    if !endpoint.path().ends_with('/') {
        let path = format!("{}/", endpoint.path());
        endpoint.set_path(&path);
    }

    match path_style {
        UrlStyle::Path => {
            // Appended as a single encoded segment, so that the name can't
            // change anything else of the url, like the host
            let path = format!("{}{}/", endpoint.path(), percent_encode(name));
            endpoint.set_path(&path);
        }
        UrlStyle::VirtualHost => {
            let host = format!("{}.{}", name, endpoint.host_str().unwrap());
            endpoint.set_host(Some(&host))?;

            // `Url::set_host` ignores anything after a `:`, lowercases the
            // host and maps or drops some other characters, which could give
            // the url of another bucket
            if endpoint.host_str() != Some(host.as_str()) {
                return Err(ParseError::InvalidDomainCharacter);
            }
        }
    }

    Ok(endpoint)
}

// === Bucket level actions ===

impl Bucket {
    /// Create a new bucket.
    ///
    /// See [`CreateBucket`] for more details.
    #[must_use]
    pub const fn create_bucket<'a>(&'a self, credentials: &'a Credentials) -> CreateBucket<'a> {
        CreateBucket::new(self, credentials)
    }

    /// Delete a bucket.
    ///
    /// See [`DeleteBucket`] for more details.
    #[must_use]
    pub const fn delete_bucket<'a>(&'a self, credentials: &'a Credentials) -> DeleteBucket<'a> {
        DeleteBucket::new(self, credentials)
    }

    /// Retrieve a bucket's policy from S3.
    ///
    /// See [`GetBucketPolicy`] for more details.
    #[cfg(feature = "full")]
    #[must_use]
    pub const fn get_bucket_policy<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
    ) -> GetBucketPolicy<'a> {
        GetBucketPolicy::new(self, credentials)
    }
}

// === Basic actions ===

impl Bucket {
    /// Retrieve an object's metadata from S3, using a `HEAD` request.
    ///
    /// See [`HeadObject`] for more details.
    #[must_use]
    pub const fn head_object<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
        object: &'a str,
    ) -> HeadObject<'a> {
        HeadObject::new(self, credentials, object)
    }

    /// Retrieve an bucket's metadata from S3, using a `HEAD` request.
    ///
    /// See [`HeadBucket`] for more details.
    #[must_use]
    pub const fn head_bucket<'a>(&'a self, credentials: Option<&'a Credentials>) -> HeadBucket<'a> {
        HeadBucket::new(self, credentials)
    }

    /// Retrieve an object from S3, using a `GET` request.
    ///
    /// See [`GetObject`] for more details.
    #[must_use]
    pub const fn get_object<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
        object: &'a str,
    ) -> GetObject<'a> {
        GetObject::new(self, credentials, object)
    }

    /// List all objects in the bucket.
    ///
    /// See [`ListObjectsV2`] for more details.
    #[cfg(feature = "xml")]
    #[must_use]
    pub fn list_objects_v2<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
    ) -> ListObjectsV2<'a> {
        ListObjectsV2::new(self, credentials)
    }

    /// Upload a file to S3, using a `PUT` request.
    ///
    /// See [`PutObject`] for more details.
    #[must_use]
    pub const fn put_object<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
        object: &'a str,
    ) -> PutObject<'a> {
        PutObject::new(self, credentials, object)
    }

    /// Delete an object from S3, using a `DELETE` request.
    ///
    /// See [`DeleteObject`] for more details.
    #[must_use]
    pub const fn delete_object<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
        object: &'a str,
    ) -> DeleteObject<'a> {
        DeleteObject::new(self, credentials, object)
    }

    /// Delete multiple objects from S3 using a single `POST` request.
    ///
    /// See [`DeleteObjects`] for more details.
    #[cfg(feature = "full")]
    pub const fn delete_objects<'a, I>(
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
    #[cfg(feature = "xml")]
    #[must_use]
    pub const fn create_multipart_upload<'a>(
        &'a self,
        credentials: Option<&'a Credentials>,
        object: &'a str,
    ) -> CreateMultipartUpload<'a> {
        CreateMultipartUpload::new(self, credentials, object)
    }

    /// Upload a part to a previously created multipart upload.
    ///
    /// See [`UploadPart`] for more details.
    #[must_use]
    pub const fn upload_part<'a>(
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
    #[cfg(feature = "xml")]
    pub const fn complete_multipart_upload<'a, I>(
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
    #[must_use]
    pub const fn abort_multipart_upload<'a>(
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
    #[cfg(feature = "xml")]
    #[must_use]
    pub const fn list_parts<'a>(
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
        match *self {
            Self::UnsupportedScheme => f.write_str("unsupported Url scheme"),
            Self::MissingHost => f.write_str("Url is missing the `host`"),
            Self::ParseError(e) => e.fmt(f),
        }
    }
}

impl StdError for BucketError {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use jiff::Timestamp;
    use pretty_assertions::assert_eq;

    use super::*;
    #[cfg(feature = "full")]
    use crate::actions::ObjectIdentifier;
    use crate::actions::S3Action;

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
    fn new_ignores_endpoint_query_and_fragment() {
        let endpoint: Url = "https://s3.dualstack.eu-west-1.amazonaws.com/?tenant=t1#frag"
            .parse()
            .unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";

        let path_style = Bucket::new(endpoint.clone(), UrlStyle::Path, name, region).unwrap();
        assert_eq!(
            path_style.base_url().as_str(),
            "https://s3.dualstack.eu-west-1.amazonaws.com/rusty-s3/"
        );

        let domain_style = Bucket::new(endpoint, UrlStyle::VirtualHost, name, region).unwrap();
        assert_eq!(
            domain_style.base_url().as_str(),
            "https://rusty-s3.s3.dualstack.eu-west-1.amazonaws.com/"
        );
    }

    #[test]
    fn new_ignores_endpoint_credentials() {
        let endpoint: Url = "https://AKIA:TOPSECRET@s3.dualstack.eu-west-1.amazonaws.com/"
            .parse()
            .unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";

        let path_style = Bucket::new(endpoint.clone(), UrlStyle::Path, name, region).unwrap();
        assert_eq!(
            path_style.base_url().as_str(),
            "https://s3.dualstack.eu-west-1.amazonaws.com/rusty-s3/"
        );

        let domain_style = Bucket::new(endpoint, UrlStyle::VirtualHost, name, region).unwrap();
        assert_eq!(
            domain_style.base_url().as_str(),
            "https://rusty-s3.s3.dualstack.eu-west-1.amazonaws.com/"
        );
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
    fn new_pathstyle_special_characters() {
        for (name, path) in [
            ("tenant:bucket", "tenant%3Abucket"),
            ("//example.com", "%2F%2Fexample.com"),
            ("http://example.com", "http%3A%2F%2Fexample.com"),
            ("a?b#c", "a%3Fb%23c"),
        ] {
            let endpoint: Url = "https://s3.dualstack.eu-west-1.amazonaws.com"
                .parse()
                .unwrap();
            let bucket = Bucket::new(endpoint, UrlStyle::Path, name, "eu-west-1").unwrap();

            assert_eq!(
                bucket.base_url().as_str(),
                format!("https://s3.dualstack.eu-west-1.amazonaws.com/{path}/")
            );
            assert_eq!(bucket.name(), name);
        }
    }

    #[test]
    fn new_domainstyle_colon() {
        let endpoint: Url = "https://s3.dualstack.eu-west-1.amazonaws.com"
            .parse()
            .unwrap();
        assert_eq!(
            Bucket::new(endpoint, UrlStyle::VirtualHost, "example.com:", "eu-west-1"),
            Err(BucketError::ParseError(ParseError::InvalidDomainCharacter))
        );
    }

    #[test]
    fn new_domainstyle_name_changed_by_host() {
        let endpoint: Url = "https://s3.dualstack.eu-west-1.amazonaws.com"
            .parse()
            .unwrap();
        // Hosts are lowercased, and other characters are mapped or dropped,
        // which would give the url of the bucket `admin`, or `admin.evil`
        for name in [
            "ADMIN",
            "\u{ff41}\u{ff44}\u{ff4d}\u{ff49}\u{ff4e}",
            "ad\u{ad}min",
            "admin\u{3002}evil",
        ] {
            assert_eq!(
                Bucket::new(endpoint.clone(), UrlStyle::VirtualHost, name, "eu-west-1"),
                Err(BucketError::ParseError(ParseError::InvalidDomainCharacter)),
                "{name}"
            );
        }
    }

    #[test]
    fn object_url_pathstyle() {
        let endpoint: Url = "https://s3.dualstack.eu-west-1.amazonaws.com"
            .parse()
            .unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        let bucket = Bucket::new(endpoint, UrlStyle::Path, name, region).unwrap();

        let path_style = bucket.object_url("something/cat.jpg");
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

        let domain_style = bucket.object_url("something/cat.jpg");
        assert_eq!(
            "https://rusty-s3.s3.dualstack.eu-west-1.amazonaws.com/something/cat.jpg",
            domain_style.as_str()
        );
    }

    #[test]
    fn path_prefix_pathstyle() {
        for endpoint in [
            "https://example.com/storage/v1/s3",
            "https://example.com/storage/v1/s3/",
        ] {
            let endpoint: Url = endpoint.parse().unwrap();
            let bucket = Bucket::new(endpoint, UrlStyle::Path, "rusty-s3", "eu-west-1").unwrap();

            assert_eq!(
                bucket.base_url().as_str(),
                "https://example.com/storage/v1/s3/rusty-s3/"
            );
            assert_eq!(
                bucket.object_url("something/cat.jpg").as_str(),
                "https://example.com/storage/v1/s3/rusty-s3/something/cat.jpg"
            );
        }
    }

    #[test]
    fn path_prefix_domainstyle() {
        for endpoint in [
            "https://example.com/storage/v1/s3",
            "https://example.com/storage/v1/s3/",
        ] {
            let endpoint: Url = endpoint.parse().unwrap();
            let bucket =
                Bucket::new(endpoint, UrlStyle::VirtualHost, "rusty-s3", "eu-west-1").unwrap();

            assert_eq!(
                bucket.base_url().as_str(),
                "https://rusty-s3.example.com/storage/v1/s3/"
            );
            assert_eq!(
                bucket.object_url("something/cat.jpg").as_str(),
                "https://rusty-s3.example.com/storage/v1/s3/something/cat.jpg"
            );
        }
    }

    #[test]
    fn object_url_leading_slashes() {
        let endpoint: Url = "https://s3.dualstack.eu-west-1.amazonaws.com"
            .parse()
            .unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        let path_bucket = Bucket::new(endpoint.clone(), UrlStyle::Path, name, region).unwrap();
        let domain_bucket = Bucket::new(endpoint, UrlStyle::VirtualHost, name, region).unwrap();

        for (object, path) in [
            ("/cat.jpg", "//cat.jpg"),
            ("/", "//"),
            ("//", "///"),
            ("//example.com/cat.jpg", "///example.com/cat.jpg"),
            ("//my cat.jpg", "///my%20cat.jpg"),
            (
                "//user@example.com:1/cat.jpg",
                "///user%40example.com%3A1/cat.jpg",
            ),
        ] {
            assert_eq!(
                path_bucket.object_url(object).as_str(),
                format!("https://s3.dualstack.eu-west-1.amazonaws.com/rusty-s3{path}")
            );
            assert_eq!(
                domain_bucket.object_url(object).as_str(),
                format!("https://rusty-s3.s3.dualstack.eu-west-1.amazonaws.com{path}")
            );
        }
    }

    /// Sign `action` at a fixed time, so that the resulting urls can be compared
    fn signed_url<'a>(action: &impl S3Action<'a>) -> Url {
        // Fri, 24 May 2013 00:00:00 GMT
        let date = Timestamp::from_second(1_369_353_600).unwrap();
        action.sign_with_time(Duration::from_secs(86400), &date)
    }

    /// Every `Bucket` helper must build the same request as the constructor of
    /// the action it returns. `object` and `upload_id` are different so that
    /// swapped arguments are caught.
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
        let creds = Some(&credentials);
        let object = "duck.jpg";
        let upload_id = "abcd";

        assert_eq!(
            signed_url(&bucket.create_bucket(&credentials)),
            signed_url(&CreateBucket::new(&bucket, &credentials))
        );
        assert_eq!(
            signed_url(&bucket.delete_bucket(&credentials)),
            signed_url(&DeleteBucket::new(&bucket, &credentials))
        );
        assert_eq!(
            signed_url(&bucket.head_bucket(creds)),
            signed_url(&HeadBucket::new(&bucket, creds))
        );
        #[cfg(feature = "full")]
        assert_eq!(
            signed_url(&bucket.get_bucket_policy(creds)),
            signed_url(&GetBucketPolicy::new(&bucket, creds))
        );

        assert_eq!(
            signed_url(&bucket.head_object(creds, object)),
            signed_url(&HeadObject::new(&bucket, creds, object))
        );
        assert_eq!(
            signed_url(&bucket.get_object(creds, object)),
            signed_url(&GetObject::new(&bucket, creds, object))
        );
        #[cfg(feature = "xml")]
        assert_eq!(
            signed_url(&bucket.list_objects_v2(creds)),
            signed_url(&ListObjectsV2::new(&bucket, creds))
        );
        assert_eq!(
            signed_url(&bucket.put_object(creds, object)),
            signed_url(&PutObject::new(&bucket, creds, object))
        );
        assert_eq!(
            signed_url(&bucket.delete_object(creds, object)),
            signed_url(&DeleteObject::new(&bucket, creds, object))
        );
        #[cfg(feature = "full")]
        {
            let objects = [ObjectIdentifier::new(object.to_owned())];
            assert_eq!(
                signed_url(&bucket.delete_objects(creds, objects.iter())),
                signed_url(&DeleteObjects::new(&bucket, creds, objects.iter()))
            );
        }

        #[cfg(feature = "xml")]
        assert_eq!(
            signed_url(&bucket.create_multipart_upload(creds, object)),
            signed_url(&CreateMultipartUpload::new(&bucket, creds, object))
        );
        assert_eq!(
            signed_url(&bucket.upload_part(creds, object, 1, upload_id)),
            signed_url(&UploadPart::new(&bucket, creds, object, 1, upload_id))
        );
        #[cfg(feature = "xml")]
        {
            let etags = ["1234"];
            assert_eq!(
                signed_url(&bucket.complete_multipart_upload(
                    creds,
                    object,
                    upload_id,
                    etags.iter().copied()
                )),
                signed_url(&CompleteMultipartUpload::new(
                    &bucket,
                    creds,
                    object,
                    upload_id,
                    etags.iter().copied()
                ))
            );
        }
        assert_eq!(
            signed_url(&bucket.abort_multipart_upload(creds, object, upload_id)),
            signed_url(&AbortMultipartUpload::new(
                &bucket, creds, object, upload_id
            ))
        );
        #[cfg(feature = "xml")]
        assert_eq!(
            signed_url(&bucket.list_parts(creds, object, upload_id)),
            signed_url(&ListParts::new(&bucket, creds, object, upload_id))
        );
    }
}
