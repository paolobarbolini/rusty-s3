use ::base64::engine::Engine as _;
use ::base64::engine::general_purpose::STANDARD;

pub(crate) fn encode<T: AsRef<[u8]>>(input: T) -> String {
    STANDARD.encode(input)
}
