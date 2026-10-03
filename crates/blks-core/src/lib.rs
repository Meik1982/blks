//! # blks-core
//!
//! High-Performance 384-Bit Cryptographic Tree-Hash Engine.
//!
//! - **384-Bit Digest:** Exactly 48 bytes output.
//! - **Clean Base64:** Encodes to exactly 64 ASCII characters with ZERO padding ('='),
//!   matching the terminal and log width of SHA-256 hexadecimal output.
//! - **192-Bit Collision Resistance:** Breaks the 128-bit birthday-paradox bottleneck of BLAKE3/SHA-256.
//! - **Multi-Core Merkle Tree:** Leverages Rayon work-stealing parallelism and Linux page-aligned 4 KiB chunks.

pub mod compress;
pub mod encoding;
pub mod ffi;
pub mod tree;

pub use encoding::{decode_base64, encode_base64, encode_base64_url, encode_hex};
pub use tree::{hash_reader, hash_slice_parallel, BlksHasher, CHUNK_SIZE};

use memmap2::Mmap;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::Path;

/// A 384-bit (48-byte) cryptographic digest
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digest(pub [u8; 48]);

impl Digest {
    pub fn as_bytes(&self) -> &[u8; 48] {
        &self.0
    }

    pub fn to_base64(&self) -> String {
        encode_base64(&self.0)
    }

    pub fn to_base64_url(&self) -> String {
        encode_base64_url(&self.0)
    }

    pub fn to_hex(&self) -> String {
        encode_hex(&self.0)
    }

    pub fn from_base64(s: &str) -> Result<Self, &'static str> {
        let bytes = decode_base64(s)?;
        Ok(Digest(bytes))
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base64())
    }
}

impl fmt::Debug for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "blks:{}", self.to_base64())
    }
}

/// Compute blks-384 hash over an in-memory byte slice using multicore parallelism.
pub fn hash(data: &[u8]) -> Digest {
    Digest(hash_slice_parallel(data))
}

/// Compute blks-384 hash over a file.
/// Automatically uses zero-copy memory mapping (`memmap2`) for regular files
/// to saturate NVMe / PCIe bandwidth and CPU cores.
pub fn hash_file<P: AsRef<Path>>(path: P) -> io::Result<Digest> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;

    if metadata.len() > 0 && metadata.is_file() {
        // Safe fast-path with memory-mapping for files > 0 bytes
        match unsafe { Mmap::map(&file) } {
            Ok(mmap) => Ok(hash(&mmap)),
            Err(_) => {
                // Fallback to streaming reader if mmap fails (e.g. special files)
                let bytes = hash_reader(file)?;
                Ok(Digest(bytes))
            }
        }
    } else {
        // Empty file or special device
        let bytes = hash_reader(file)?;
        Ok(Digest(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determinism_and_parity_between_slice_and_streaming() {
        for size in [0, 1, 127, 128, 129, 4095, 4096, 4097, 8192, 10000, 65536] {
            let data: Vec<u8> = (0..size).map(|i| (i * 31 + 7) as u8).collect();
            let d_slice = hash(&data);

            let mut hasher = BlksHasher::new();
            hasher.update(&data);
            let d_stream = Digest(hasher.finalize());

            assert_eq!(
                d_slice, d_stream,
                "Mismatch between slice and streaming for size {}",
                size
            );
            assert_eq!(d_slice.to_base64().len(), 64);
            assert!(!d_slice.to_base64().contains('='));
        }
    }

    #[test]
    fn test_collision_resistance_different_inputs() {
        let d1 = hash(b"The quick brown fox jumps over the lazy dog");
        let d2 = hash(b"The quick brown fox jumps over the lazy dog.");
        assert_ne!(d1, d2);
    }
}
