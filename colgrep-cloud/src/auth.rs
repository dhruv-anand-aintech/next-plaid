//! Password hashing using PBKDF2-SHA256 (pure Rust, WASM-compatible)

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

const SALT_LEN: usize = 16;
const PBKDF2_ITERATIONS: u32 = 100_000;
const HASH_LEN: usize = 32;

/// Hash password with random salt. Pass random bytes from worker::crypto::get_random_values.
pub fn hash_password(password: &str, salt: &[u8; SALT_LEN]) -> String {
    let mut hash = [0u8; HASH_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut hash);

    let mut encoded = Vec::with_capacity(SALT_LEN + HASH_LEN);
    encoded.extend_from_slice(salt);
    encoded.extend_from_slice(&hash);
    BASE64.encode(&encoded)
}

/// Verify password against stored hash
pub fn verify_password(password: &str, stored: &str) -> Result<bool, String> {
    let decoded = BASE64
        .decode(stored)
        .map_err(|_| "Invalid stored hash".to_string())?;
    if decoded.len() != SALT_LEN + HASH_LEN {
        return Ok(false);
    }

    let (salt, _) = decoded.split_at(SALT_LEN);
    let mut hash = [0u8; HASH_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut hash);

    Ok(decoded[SALT_LEN..] == hash)
}
