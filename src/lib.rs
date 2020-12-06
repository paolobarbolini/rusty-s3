//! A simple, pure Rust AWS S3 client following a Sans-IO approach
//!
//! The rusty-s3 crate provides a convenient API for signing, building
//! and parsing AWS S3 requests and responses.
//! It follows a Sans-IO approach, meaning that the library itself doesn't
//! send any of the requests. It's the reposibility of the user to choose an
//! HTTP client, be it synchronous or asynchronous, and use it to send the requests.
//!
//! ## Building a Bucket and Credentials
//!
//! ```rust
//! use std::env;
//! use rusty_s3::{Bucket, Credentials};
//! # env::set_var("AWS_ACCESS_KEY_ID", "key");
//! # env::set_var("AWS_SECRET_ACCESS_KEY", "secret");
//!
//! let endpoint = "https://eu-west-1.s3.amazonaws.com".parse().expect("endpoint is a valid Url");
//! let path_style = true;
//! let name = String::from("rusty-s3");
//! let region = String::from("eu-west-1");
//! let bucket = Bucket::new(endpoint, path_style, name, region).expect("Url has a valid scheme and host");
//!
//! let key = env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID is set and a valid String");
//! let secret = env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_ACCESS_KEY_ID is set and a valid String");
//! let credentials = Credentials::new(key, secret);
//! ```

#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    broken_intra_doc_links
)]

pub use self::bucket::Bucket;
pub use self::credentials::Credentials;
pub use self::map::Map;

pub mod actions;
mod bucket;
mod credentials;
mod map;
mod signing;
pub(crate) mod sorting_iter;
