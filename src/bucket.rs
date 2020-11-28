use url::{ParseError, Url};

use crate::signing::util::percent_encode_path;

pub struct Bucket {
    endpoint: Url,
    name: String,
    region: String,
}

impl Bucket {
    pub fn new(endpoint: Url, name: String, region: String) -> Option<Self> {
        let _ = endpoint.host_str()?;

        match endpoint.scheme() {
            "http" | "https" => {}
            _ => return None,
        };

        Some(Self {
            endpoint,
            name,
            region,
        })
    }

    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    fn base_url(&self, path_style: bool) -> Url {
        if path_style {
            let path = format!("{}/", self.name);
            self.endpoint.join(&path).unwrap()
        } else {
            let mut url = self.endpoint.clone();

            let host = format!("{}.{}", self.name, url.host_str().unwrap());
            url.set_host(Some(&host)).unwrap();

            url
        }
    }

    pub fn object_url(&self, path_style: bool, object: &str) -> Result<Url, ParseError> {
        let base_url = self.base_url(path_style);

        let object = percent_encode_path(object);
        base_url.join(&object)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let endpoint: Url = "https://s3.eu-west-1.amazonaws.com".parse().unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        let bucket = Bucket::new(endpoint.clone(), name.into(), region.into()).unwrap();

        assert_eq!(bucket.endpoint(), &endpoint);
        assert_eq!(bucket.name(), name);
        assert_eq!(bucket.region(), region);
    }

    #[test]
    fn new_bad_scheme() {
        let endpoint = "file:///home/something".parse().unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        assert!(Bucket::new(endpoint, name.into(), region.into()).is_none());
    }

    #[test]
    fn object_url() {
        let endpoint: Url = "https://s3.eu-west-1.amazonaws.com".parse().unwrap();
        let name = "rusty-s3";
        let region = "eu-west-1";
        let bucket = Bucket::new(endpoint.clone(), name.into(), region.into()).unwrap();

        let path_style = bucket.object_url(true, "something/cat.jpg").unwrap();
        let domain_style = bucket.object_url(false, "something/cat.jpg").unwrap();
        assert_eq!(
            "https://s3.eu-west-1.amazonaws.com/rusty-s3/something/cat.jpg",
            path_style.as_str()
        );
        assert_eq!(
            "https://rusty-s3.s3.eu-west-1.amazonaws.com/something/cat.jpg",
            domain_style.as_str()
        );
    }
}
