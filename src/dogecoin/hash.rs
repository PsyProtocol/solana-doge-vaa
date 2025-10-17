

pub type QHash256 = [u8; 32];
pub type QHash160 = [u8; 20];


pub trait DogeHashProvider {
    fn hash_bytes_sha256(data: &[u8]) -> QHash256;
    fn hash_bytes_ripemd160(data: &[u8]) -> QHash160;

    // performs ripemd160(sha256(data))
    fn bitcoin_hash160(data: &[u8]) -> QHash160 {
        let sha256_hash = Self::hash_bytes_sha256(data);
        Self::hash_bytes_ripemd160(&sha256_hash)
    }

    // performs sha256(sha256(data))
    fn bitcoin_hash256(data: &[u8]) -> QHash256 {
        let first_hash = Self::hash_bytes_sha256(data);
        Self::hash_bytes_sha256(&first_hash)
    }

}


#[cfg(feature = "hashes")]
use sha2::{Digest as Sha256Digest, Sha256};

#[cfg(feature = "hashes")]
use ripemd::Ripemd160;

#[cfg(feature = "hashes")]
pub struct CommonDogeHashProvider;

#[cfg(feature = "hashes")]
impl DogeHashProvider for CommonDogeHashProvider {
    fn hash_bytes_sha256(data: &[u8]) -> QHash256 {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        result.into()
    }
    fn hash_bytes_ripemd160(data: &[u8]) -> QHash160 {
        let mut hasher = Ripemd160::new();
        hasher.update(data);
        let result = hasher.finalize();
        result.into()
    }
}
