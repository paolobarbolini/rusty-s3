/// The format of the time of the signature, like in `X-Amz-Date`.
pub(crate) const ISO8601: &str = "%Y%m%dT%H%M%SZ";

/// The format of the date in the scope of the signature, like in
/// `X-Amz-Credential`.
pub(crate) const YYYYMMDD: &str = "%Y%m%d";
