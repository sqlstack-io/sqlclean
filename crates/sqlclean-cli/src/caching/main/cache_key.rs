//! The digest that identifies one cached result.

use sha2::{Digest, Sha256};

/// A SHA-256 over this build's version and every part, each prefixed by its length.
pub fn cache_key(parts: &[&str]) -> String {
    let mut digest = Sha256::new();
    for part in [env!("CARGO_PKG_VERSION")].iter().chain(parts) {
        digest.update(part.len().to_string().as_bytes());
        digest.update(b":");
        digest.update(part.as_bytes());
    }
    format!("{:x}", digest.finalize())
}
