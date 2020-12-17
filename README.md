# rusty-s3

[![crates.io](https://img.shields.io/crates/v/rusty-s3.svg)](https://crates.io/crates/rusty-s3)
[![Documentation](https://docs.rs/rusty-s3/badge.svg)](https://docs.rs/rusty-s3)
[![dependency status](https://deps.rs/crate/rusty-s3/0.0.2/status.svg)](https://deps.rs/crate/rusty-s3/0.0.2)
[![Rustc Version 1.42.0+](https://img.shields.io/badge/rustc-1.42.0+-lightgray.svg)](https://blog.rust-lang.org/2020/03/12/Rust-1.42.html)
[![CI](https://github.com/paolobarbolini/rusty-s3/workflows/CI/badge.svg)](https://github.com/paolobarbolini/rusty-s3/actions?query=workflow%3ACI)
[![codecov](https://codecov.io/gh/paolobarbolini/rusty-s3/branch/main/graph/badge.svg?token=K0YPC21N8D)](https://codecov.io/gh/paolobarbolini/rusty-s3)

Simple pure Rust AWS S3 Client following a Sans-IO approach, with a modern
and rusty take onto s3's APIs.

Request signing and response parsing capabilities are provided for the
most commons S3 actions.

See https://docs.rs/rusty-s3 or look at the `examples` folder for more examples.

Minio compatibility tested on every commit by GitHub Actions.

## Supported S3 actions

* Bucket level methods
    * [`CreateBucket`][createbucket]
* Basic methods
    * [`GetObject`][getobject]
    * [`PutObject`][putobject]
    * [`DeleteObject`][deleteobject]
    * [`ListObjectsV2`][listobjectsv2]
* Multipart upload
    * [`CreateMultipartUpload`][completemultipart]
    * [`UploadPart`][uploadpart]
    * [`CompleteMultipartUpload`][completemultipart]
    * [`AbortMultipartUpload`][abortmultipart]

[abortmultipart]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_AbortMultipartUpload.html
[completemultipart]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html
[createbucket]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateBucket.html
[createmultipart]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html
[deleteobject]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObject.html
[getobject]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html
[listobjectsv2]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html
[putobject]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html
[uploadpart]: https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPart
