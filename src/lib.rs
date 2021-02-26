//! A simple, pure Rust AWS S3 client following a Sans-IO approach
//!
//! The rusty-s3 crate provides a convenient API for signing, building
//! and parsing AWS S3 requests and responses.
//! It follows a Sans-IO approach, meaning that the library itself doesn't
//! send any of the requests. It's the reposibility of the user to choose an
//! HTTP client, be it synchronous or asynchronous, and use it to send the requests.
//!
//! ## Basic getting started example
//!
//! ```rust
//! use std::env;
//! use std::time::Duration;
//! use rusty_s3::{Bucket, Credentials, S3Action};
//! # env::set_var("AWS_ACCESS_KEY_ID", "key");
//! # env::set_var("AWS_SECRET_ACCESS_KEY", "secret");
//!
//! // setting up a bucket
//! let endpoint = "https://s3-eu-west-1.amazonaws.com".parse().expect("endpoint is a valid Url");
//! let path_style = true;
//! let name = String::from("rusty-s3");
//! let region = String::from("eu-west-1");
//! let bucket = Bucket::new(endpoint, path_style, name, region).expect("Url has a valid scheme and host");
//!
//! // setting up the credentials
//! let key = env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID is set and a valid String");
//! let secret = env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_ACCESS_KEY_ID is set and a valid String");
//! let credentials = Credentials::new(key, secret);
//!
//! // signing a request
//! let presigned_url_duration = Duration::from_secs(60 * 60);
//! let action = bucket.get_object(Some(&credentials), "duck.jpg");
//! println!("GET {}", action.sign(presigned_url_duration));
//! ```

#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    broken_intra_doc_links
)]
#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/rusty-s3/0.1.2")]

pub use self::actions::S3Action;
pub use self::bucket::Bucket;
pub use self::credentials::Credentials;
pub use self::map::Map;
pub use self::method::Method;

pub mod actions;
mod bucket;
pub mod credentials;
mod map;
mod method;
mod signing;
pub(crate) mod sorting_iter;
