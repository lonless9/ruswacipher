use crate::error::Result;

/// Result of an encryption operation containing the IV and ciphertext
#[derive(Debug, Clone)]
pub struct EncryptionResult {
    pub iv: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

impl EncryptionResult {
    /// Serialize the encryption result by prepending IV to ciphertext
    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.iv.len() + self.ciphertext.len());
        result.extend_from_slice(&self.iv);
        result.extend_from_slice(&self.ciphertext);
        result
    }

    /// Deserialize data by splitting IV and ciphertext
    pub fn deserialize(data: &[u8], iv_length: usize) -> Result<Self> {
        if data.len() < iv_length {
            return Err(crate::error::RusWaCipherError::Decryption(
                "Data too short to contain IV".to_string(),
            ));
        }

        let (iv, ciphertext) = data.split_at(iv_length);
        Ok(EncryptionResult {
            iv: iv.to_vec(),
            ciphertext: ciphertext.to_vec(),
        })
    }
}

/// Trait for encryption/decryption operations
pub trait Cipher {
    /// Encrypt data and return IV + ciphertext
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptionResult>;

    /// Decrypt data using provided IV and ciphertext
    fn decrypt(&self, iv: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>>;

    /// Get the IV length for this cipher
    fn iv_length(&self) -> usize;

    /// Get the key length for this cipher
    fn key_length(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_result_serialize() {
        let result = EncryptionResult {
            iv: vec![1, 2, 3, 4],
            ciphertext: vec![5, 6, 7, 8, 9],
        };

        let serialized = result.serialize();
        assert_eq!(serialized, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_encryption_result_deserialize() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        let iv_length = 4;

        let result = EncryptionResult::deserialize(&data, iv_length).unwrap();
        assert_eq!(result.iv, vec![1, 2, 3, 4]);
        assert_eq!(result.ciphertext, vec![5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_encryption_result_deserialize_empty_ciphertext() {
        let data = vec![1, 2, 3, 4];
        let iv_length = 4;

        let result = EncryptionResult::deserialize(&data, iv_length).unwrap();
        assert_eq!(result.iv, vec![1, 2, 3, 4]);
        assert_eq!(result.ciphertext, vec![]);
    }

    #[test]
    fn test_encryption_result_deserialize_too_short() {
        let data = vec![1, 2, 3];
        let iv_length = 4;

        let result = EncryptionResult::deserialize(&data, iv_length);
        assert!(result.is_err());
    }

    #[test]
    fn test_encryption_result_round_trip() {
        let original = EncryptionResult {
            iv: vec![
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
            ],
            ciphertext: vec![0xFF, 0xFE, 0xFD, 0xFC, 0xFB],
        };

        let serialized = original.serialize();
        let deserialized = EncryptionResult::deserialize(&serialized, 12).unwrap();

        assert_eq!(original.iv, deserialized.iv);
        assert_eq!(original.ciphertext, deserialized.ciphertext);
    }

    #[test]
    fn test_encryption_result_clone() {
        let result = EncryptionResult {
            iv: vec![1, 2, 3],
            ciphertext: vec![4, 5, 6],
        };

        let cloned = result.clone();
        assert_eq!(result.iv, cloned.iv);
        assert_eq!(result.ciphertext, cloned.ciphertext);
    }
}
