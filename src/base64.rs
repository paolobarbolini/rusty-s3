use ::base64::engine::general_purpose::STANDARD;
use ::base64::engine::Engine;

pub(crate) fn encode<T: AsRef<[u8]>>(input: T) -> String {
    STANDARD.encode(input)
}
