use std::fmt::{self, Display};

/// The HTTP request method for an [`S3Action`](crate::actions::S3Action).
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Method {
    Head,
    Get,
    Post,
    Put,
    Delete,
}

impl Method {
    /// Convert this `Method` into an uppercase string.
    ///
    /// ```rust
    /// # use rusty_s3::Method;
    /// assert_eq!(Method::Get.to_str(), "GET");
    /// ```
    #[inline]
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Head => "HEAD",
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
        }
    }
}

impl Display for Method {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_str() {
        assert_eq!(Method::Head.to_str(), "HEAD");
        assert_eq!(Method::Get.to_str(), "GET");
        assert_eq!(Method::Post.to_str(), "POST");
        assert_eq!(Method::Put.to_str(), "PUT");
        assert_eq!(Method::Delete.to_str(), "DELETE");
    }

    #[test]
    fn display() {
        assert_eq!(Method::Head.to_string(), "HEAD");
        assert_eq!(Method::Get.to_string(), "GET");
        assert_eq!(Method::Post.to_string(), "POST");
        assert_eq!(Method::Put.to_string(), "PUT");
        assert_eq!(Method::Delete.to_string(), "DELETE");
    }
}
