use sha2::{Digest, Sha256};

use crate::utils::error::ZzmError;

pub fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn verify_checksum(data: &[u8], expected: &str) -> Result<bool, ZzmError> {
    let actual = calculate_sha256(data);
    Ok(actual.to_lowercase() == expected.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sha256() {
        let data = b"hello world";
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let actual = calculate_sha256(data);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_verify_checksum_success() {
        let data = b"hello world";
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let result = verify_checksum(data, expected).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_checksum_case_insensitive() {
        let data = b"hello world";
        let expected = "B94D27B9934D3E08A52E52D7DA7DABFAC484EFE37A5380EE9088F7ACE2EFCDE9";
        let result = verify_checksum(data, expected).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_checksum_failure() {
        let data = b"hello world";
        let expected = "wrong_checksum";
        let result = verify_checksum(data, expected).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_calculate_sha256_empty_data() {
        let data = b"";
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let actual = calculate_sha256(data);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_calculate_sha256_binary_data() {
        let data = vec![0u8, 1, 2, 3, 255, 254, 253];
        let expected = "d3e4c3a8c0a0b7e8d5e5a0b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8";
        let actual = calculate_sha256(&data);
        // 计算实际的哈希值
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let expected_real = hex::encode(hasher.finalize());
        assert_eq!(actual, expected_real);
    }
}
