use url::{ParseError, Url};

use crate::signing::util::percent_encode_path;

pub struct Bucket {
    base_url: Url,
    name: String,
    region: String,
}

impl Bucket {
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

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn region(&self) -> &str {
        &self.region
    }

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
