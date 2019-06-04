use base64::{URL_SAFE_NO_PAD};
use rand::{Rng};

pub struct SecureRandomBase64;

impl SecureRandomBase64  {
    pub fn generate() -> String {
        let random_bytes = rand::thread_rng()
            .gen::<[u8; 12]>();

        let tok = base64::encode_config(
            &random_bytes,
            URL_SAFE_NO_PAD
        );

        tok
    }
}