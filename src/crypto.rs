#[cfg(not(any(feature = "aws-lc-rs", feature = "rustcrypto", feature = "graviola")))]
compile_error!(
    "enable at least one of `aws-lc-rs`, `rustcrypto`, or `graviola` to provide a crypto backend"
);

/// Compute the SHA-256 digest of `data`.
pub(crate) fn sha256(data: &[u8]) -> [u8; 32] {
    #[cfg(feature = "aws-lc-rs")]
    {
        use aws_lc_rs::digest::{SHA256, digest};
        digest(&SHA256, data).as_ref().try_into().unwrap()
    }

    #[cfg(all(feature = "rustcrypto", not(feature = "aws-lc-rs")))]
    {
        use sha2::Digest as _;
        sha2::Sha256::digest(data).into()
    }

    #[cfg(not(any(feature = "aws-lc-rs", feature = "rustcrypto")))]
    {
        use graviola::hashing::Hash as _;
        graviola::hashing::Sha256::hash(data)
            .as_ref()
            .try_into()
            .unwrap()
    }
}

/// Compute the SHA-256 HMAC digest of `data`.
pub(crate) fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    #[cfg(feature = "aws-lc-rs")]
    {
        use aws_lc_rs::hmac::{self, HMAC_SHA256, Key};
        hmac::sign(&Key::new(HMAC_SHA256, key), data)
            .as_ref()
            .try_into()
            .unwrap()
    }

    #[cfg(all(feature = "rustcrypto", not(feature = "aws-lc-rs")))]
    {
        use hmac::{KeyInit as _, Mac as _};
        let mut hmac = hmac::Hmac::<sha2::Sha256>::new_from_slice(key)
            .expect("HMAC-SHA256 accepts any key length >= 1");
        hmac.update(data);
        hmac.finalize().into_bytes().into()
    }

    #[cfg(not(any(feature = "aws-lc-rs", feature = "rustcrypto")))]
    {
        let mut hmac = graviola::hashing::hmac::Hmac::<graviola::hashing::Sha256>::new(key);
        hmac.update(data);
        hmac.finish().as_ref().try_into().unwrap()
    }
}
