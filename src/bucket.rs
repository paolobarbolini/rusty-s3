use url::{ParseError, Url};

use crate::signing::util::percent_encode_path;

/// An S3 bucket
///
/// ## Path style url
///
/// ```rust
/// # use rusty_s3::Bucket;
/// let endpoint = "https://eu-west-1.s3.amazonaws.com".parse().expect("endpoint is a valid Url");
/// let path_style = true;
/// let name = String::from("rusty-s3");
/// let region = String::from("eu-west-1");
///
/// let bucket = Bucket::new(endpoint, path_style, name, region).expect("Url has a valid scheme and host");
/// assert_eq!(bucket.base_url().as_str(), "https://eu-west-1.s3.amazonaws.com/rusty-s3/");
/// assert_eq!(bucket.name(), "rusty-s3");
/// assert_eq!(bucket.region(), "eu-west-1");
/// assert_eq!(bucket.object_url("duck.jpg").expect("url is valid").as_str(), "https://eu-west-1.s3.amazonaws.com/rusty-s3/duck.jpg");
/// ```
///
/// ## Domain style url
///
/// ```rust
/// # use rusty_s3::Bucket;
/// let endpoint = "https://eu-west-1.s3.amazonaws.com".parse().expect("endpoint is a valid Url");
/// let path_style = false;
/// let name = String::from("rusty-s3");
/// let region = String::from("eu-west-1");
///
/// let bucket = Bucket::new(endpoint, path_style, name, region).expect("Url has a valid scheme and host");
/// assert_eq!(bucket.base_url().as_str(), "https://rusty-s3.eu-west-1.s3.amazonaws.com/");
/// assert_eq!(bucket.name(), "rusty-s3");
/// assert_eq!(bucket.region(), "eu-west-1");
/// assert_eq!(bucket.object_url("duck.jpg").expect("url is valid").as_str(), "https://rusty-s3.eu-west-1.s3.amazonaws.com/duck.jpg");
/// ```
#[derive(Debug, Clone)]
pub struct Bucket {
    base_url: Url,
    name: String,
    region: String,
}

impl Bucket {
    /// Construct a new S3 bucket
    pub fn new(endpoint: Url, path_style: bool, name: String, region: String) -> Option<Self> {
        let _ = endpoint.host_str()?;

        match endpoint.scheme() {
            "http" | "https" => {}
            _ => return None,
        };

        let base_url = base_url(endpoint, &name, path_style);

        Some(Self {
            base_url,
            name,
            region,
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

fn base_url(mut endpoint: Url, name: &str, path_style: bool) -> Url {
    if path_style {
        let path = format!("{}/", name);
        endpoint.join(&path).unwrap()
    } else {
        let host = format!("{}.{}", name, endpoint.host_str().unwrap());
        endpoint.set_host(Some(&host)).unwrap();
        endpoint
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn new_pathstyle() {
        let endpoint: Url = "https://s3.eu-west-1.amazonaws.com".parse().unwrap();
        let base_url: Url = "https://s3.eu-west-1.amazonaws.com/rusty-s3/"
            .parse()
            .unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        let bucket = Bucket::new(endpoint, true, name.into(), region.into()).unwrap();

        assert_eq!(bucket.base_url(), &base_url);
        assert_eq!(bucket.name(), name);
        assert_eq!(bucket.region(), region);
    }

    #[test]
    fn new_domainstyle() {
        let endpoint: Url = "https://s3.eu-west-1.amazonaws.com".parse().unwrap();
        let base_url: Url = "https://rusty-s3.s3.eu-west-1.amazonaws.com"
            .parse()
            .unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        let bucket = Bucket::new(endpoint, false, name.into(), region.into()).unwrap();

        assert_eq!(bucket.base_url(), &base_url);
        assert_eq!(bucket.name(), name);
        assert_eq!(bucket.region(), region);
    }

    #[test]
    fn new_bad_scheme() {
        let endpoint = "file:///home/something".parse().unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        assert!(Bucket::new(endpoint, true, name.into(), region.into()).is_none());
    }

    #[test]
    fn object_url_pathstyle() {
        let endpoint: Url = "https://s3.eu-west-1.amazonaws.com".parse().unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        let bucket = Bucket::new(endpoint, true, name.into(), region.into()).unwrap();

        let path_style = bucket.object_url("something/cat.jpg").unwrap();
        assert_eq!(
            "https://s3.eu-west-1.amazonaws.com/rusty-s3/something/cat.jpg",
            path_style.as_str()
        );
    }

    #[test]
    fn object_url_domainstyle() {
        let endpoint: Url = "https://s3.eu-west-1.amazonaws.com".parse().unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        let bucket = Bucket::new(endpoint, false, name.into(), region.into()).unwrap();

        let domain_style = bucket.object_url("something/cat.jpg").unwrap();
        assert_eq!(
            "https://rusty-s3.s3.eu-west-1.amazonaws.com/something/cat.jpg",
            domain_style.as_str()
        );
    }
}
