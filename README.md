# rusty-s3

[![crates.io](https://img.shields.io/crates/v/rusty-s3.svg)](https://crates.io/crates/rusty-s3)
[![Documentation](https://docs.rs/rusty-s3/badge.svg)](https://docs.rs/rusty-s3)
[![dependency status](https://deps.rs/crate/rusty-s3/0.5.0/status.svg)](https://deps.rs/crate/rusty-s3/0.5.0)
[![Rustc Version 1.67+](https://img.shields.io/badge/rustc-1.67+-lightgray.svg)](https://blog.rust-lang.org/2023/01/26/Rust-1.67.0.html)
[![CI](https://github.com/paolobarbolini/rusty-s3/workflows/CI/badge.svg)](https://github.com/paolobarbolini/rusty-s3/actions?query=workflow%3ACI)
[![codecov](https://codecov.io/gh/paolobarbolini/rusty-s3/branch/main/graph/badge.svg?token=K0YPC21N8D)](https://codecov.io/gh/paolobarbolini/rusty-s3)

Simple pure Rust AWS S3 Client following a Sans-IO approach, with a modern
and rusty take onto s3's APIs.

Request signing and response parsing capabilities are provided for the
most common S3 actions, using AWS Signature Version 4.

Minio compatibility tested on every commit by GitHub Actions.

## Examples

```rust
use std::env;
use std::time::Duration;
use rusty_s3::{Bucket, Credentials, S3Action, UrlStyle};

// setting up a bucket
let endpoint = "https://s3.dualstack.eu-west-1.amazonaws.com".parse().expect("endpoint is a valid Url");
let path_style = UrlStyle::VirtualHost;
let name = "rusty-s3";
let region = "eu-west-1";
let bucket = Bucket::new(endpoint, path_style, name, region).expect("Url has a valid scheme and host");

// setting up the credentials
let key = env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID is set and a valid String");
let secret = env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_ACCESS_KEY_ID is set and a valid String");
let credentials = Credentials::new(key, secret);

// signing a request
let presigned_url_duration = Duration::from_secs(60 * 60);
let action = bucket.get_object(Some(&credentials), "duck.jpg");
println!("GET {}", action.sign(presigned_url_duration));
```

More examples can be found in the examples directory on GitHub.

## Supported S3 actions

* Bucket level methods
    * [`CreateBucket`][createbucket]
    * [`DeleteBucket`][deletebucket]
    * [`HeadBucket`][headbucket]
* Basic methods
    * [`HeadObject`][headobject]
    * [`GetObject`][getobject]
    * [`PutObject`][putobject]
    * [`DeleteObject`][deleteobject]
    * [`DeleteObjects`][deleteobjects]
    * [`ListObjectsV2`][listobjectsv2]
* Multipart upload
    * [`CreateMultipartUpload`][completemultipart]
    * [`UploadPart`][uploadpart]
    * [`ListParts`][listparts]
    * [`CompleteMultipartUpload`][completemultipart]
    * [`AbortMultipartUpload`][abortmultipart]

[abortmultipart]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_AbortMultipartUpload.html
[completemultipart]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html
[listparts]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListParts.html
[createbucket]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateBucket.html
[deletebucket]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucket.html
[headbucket]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadBucket.html
[createmultipart]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html
[deleteobject]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObject.html
[deleteobjects]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjects.html
[getobject]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html
[headobject]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadObject.html
[listobjectsv2]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html
[putobject]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html
[uploadpart]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPart
