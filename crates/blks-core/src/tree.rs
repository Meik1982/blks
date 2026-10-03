//! High-Performance Merkle Tree Engine for blks-384
//!
//! Supports both parallel tree reduction (via rayon) for in-memory slices and memory-mapped files,
//! as well as streaming reduction for sequential I/O (stdin, pipes).

use crate::compress::{
    compress, compress_parent, initial_cv, BLOCK_BYTES, FLAG_CHUNK_END, FLAG_CHUNK_START, FLAG_ROOT,
};
use rayon::prelude::*;
use std::io::{self, Read};

/// Standard chunk size for leaf nodes: 4096 bytes (4 KiB, matching Linux page size)
pub const CHUNK_SIZE: usize = 4096;

/// Hash a single chunk (up to 4096 bytes).
/// If `is_root` is true, this chunk is the entire input and receives FLAG_ROOT.
pub fn hash_chunk(chunk: &[u8], is_root: bool) -> [u8; 48] {
    let len = chunk.len();
    assert!(len <= CHUNK_SIZE);

    let num_blocks = if len == 0 {
        1
    } else {
        len.div_ceil(BLOCK_BYTES)
    };

    let mut cv = initial_cv();
    let mut block = [0u8; BLOCK_BYTES];

    for i in 0..num_blocks {
        let start = i * BLOCK_BYTES;
        let end = (start + BLOCK_BYTES).min(len);
        let block_len = if start < len { end - start } else { 0 };

        block.fill(0);
        if block_len > 0 {
            block[..block_len].copy_from_slice(&chunk[start..end]);
        }

        let mut flags = 0u64;
        if i == 0 {
            flags |= FLAG_CHUNK_START;
        }
        if i == num_blocks - 1 {
            flags |= FLAG_CHUNK_END;
            if is_root {
                flags |= FLAG_ROOT;
            }
        }

        cv = compress(&cv, &block, end as u64, flags);
    }

    let mut out = [0u8; 48];
    for i in 0..6 {
        out[i * 8..(i + 1) * 8].copy_from_slice(&cv[i].to_le_bytes());
    }
    out
}

/// Level-by-level binary Merkle tree reduction.
/// Guarantees bit-exact identical tree structures across multithreaded and streaming paths.
pub fn reduce_nodes(mut current_level: Vec<[u8; 48]>) -> [u8; 48] {
    assert!(!current_level.is_empty(), "cannot reduce empty tree");
    if current_level.len() == 1 {
        return current_level[0];
    }

    while current_level.len() > 1 {
        let is_root_level = current_level.len() == 2;
        let num_parents = current_level.len().div_ceil(2);
        let mut next_level = Vec::with_capacity(num_parents);

        let mut i = 0;
        while i < current_level.len() {
            if i + 1 < current_level.len() {
                let parent =
                    compress_parent(&current_level[i], &current_level[i + 1], is_root_level);
                next_level.push(parent);
                i += 2;
            } else {
                // Orphaned right-hand node: promote directly to the next level
                next_level.push(current_level[i]);
                i += 1;
            }
        }
        current_level = next_level;
    }

    current_level[0]
}

/// Compute blks-384 hash over an in-memory slice using Rayon multithreading.
pub fn hash_slice_parallel(data: &[u8]) -> [u8; 48] {
    if data.len() <= CHUNK_SIZE {
        return hash_chunk(data, true);
    }

    let leaves: Vec<[u8; 48]> = data
        .par_chunks(CHUNK_SIZE)
        .map(|chunk| hash_chunk(chunk, false))
        .collect();

    reduce_nodes(leaves)
}

/// Incremental streaming hasher for blks-384 (e.g. stdin or network streams).
/// Produces bit-exact identical hashes to `hash_slice_parallel`.
pub struct BlksHasher {
    buffer: Vec<u8>,
    leaf_hashes: Vec<[u8; 48]>,
    total_bytes: u64,
}

impl Default for BlksHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl BlksHasher {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(CHUNK_SIZE),
            leaf_hashes: Vec::new(),
            total_bytes: 0,
        }
    }

    pub fn update(&mut self, mut input: &[u8]) {
        self.total_bytes += input.len() as u64;

        while !input.is_empty() {
            if self.buffer.len() == CHUNK_SIZE {
                let leaf_hash = hash_chunk(&self.buffer, false);
                self.leaf_hashes.push(leaf_hash);
                self.buffer.clear();
            }

            let available = CHUNK_SIZE - self.buffer.len();
            let to_take = available.min(input.len());
            self.buffer.extend_from_slice(&input[..to_take]);
            input = &input[to_take..];
        }
    }

    pub fn finalize(mut self) -> [u8; 48] {
        if self.leaf_hashes.is_empty() {
            // Entire input fits in single chunk buffer (<= 4096 bytes)
            return hash_chunk(&self.buffer, true);
        }

        if !self.buffer.is_empty() {
            let final_leaf = hash_chunk(&self.buffer, false);
            self.leaf_hashes.push(final_leaf);
        }

        reduce_nodes(self.leaf_hashes)
    }
}

/// Helper function to hash an `io::Read` stream to completion
pub fn hash_reader<R: Read>(mut reader: R) -> io::Result<[u8; 48]> {
    let mut hasher = BlksHasher::new();
    let mut buf = vec![0u8; 64 * 1024]; // 64 KiB I/O buffer
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize())
}
