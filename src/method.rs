use std::fmt::{self, Display};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Method {
    Head,
    Get,
    Post,
    Put,
    Delete,
}

impl Method {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_str())
    }
}
