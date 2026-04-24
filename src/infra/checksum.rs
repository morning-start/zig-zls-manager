use sha2::{Digest, Sha256};

pub fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn verify_checksum(data: &[u8], expected: &str) -> Result<bool, crate::utils::ZzmError> {
    let actual = calculate_sha256(data);
    Ok(actual.to_lowercase() == expected.to_lowercase())
}
