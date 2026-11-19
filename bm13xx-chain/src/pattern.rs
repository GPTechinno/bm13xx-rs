//! Known-nonce tests.
//!
//! A block header whose nonce is known makes a test job: the chip whose share of the nonce range
//! holds that nonce must return it, at the base version, soon after the job is sent. Run them
//! with [`Chain::known_nonce_test`](crate::Chain::known_nonce_test). A header tests one core of one
//! chip; covering a whole chain takes many headers, such as consecutive block headers of the
//! Bitcoin blockchain. With version rolling, pick headers whose version has no bit in the version
//! rolling mask, so the nonce is found at the base version.

use crate::MAX_ASIC_CNT;
use heapless::Vec;

/// # A block header with a known nonce
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct KnownNonce {
    pub version: u32,
    /// Previous block hash, in display order.
    pub prev_block_header_hash: [u8; 32],
    /// Merkle root, in display order.
    pub merkle_root: [u8; 32],
    pub n_bits: u32,
    pub n_time: u32,
    pub nonce: u32,
}

/// The first three blocks of the Bitcoin blockchain.
pub const BITCOIN_BLOCKS_0_TO_2: [KnownNonce; 3] = [
    KnownNonce {
        version: 1,
        prev_block_header_hash: [0; 32],
        merkle_root: hex32(b"4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"),
        n_bits: 0x1d00_ffff,
        n_time: 1_231_006_505,
        nonce: 2_083_236_893,
    },
    KnownNonce {
        version: 1,
        prev_block_header_hash: hex32(
            b"000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
        ),
        merkle_root: hex32(b"0e3e2357e806b6cdb1f70b54c3a3a17b6714ee1f0e68bebb44a74b1efd512098"),
        n_bits: 0x1d00_ffff,
        n_time: 1_231_469_665,
        nonce: 2_573_394_689,
    },
    KnownNonce {
        version: 1,
        prev_block_header_hash: hex32(
            b"00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048",
        ),
        merkle_root: hex32(b"9b0fc92260312ce44e74ef369f5c66bbb85848f2eddd5a7a1cde251e54ccfdd5"),
        n_bits: 0x1d00_ffff,
        n_time: 1_231_469_744,
        nonce: 1_639_830_024,
    },
];

/// 32 bytes from 64 hexadecimal digits.
const fn hex32(hex: &[u8; 64]) -> [u8; 32] {
    const fn digit(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => core::panic!("not a hexadecimal digit"),
        }
    }
    let mut out = [0; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = digit(hex[2 * i]) << 4 | digit(hex[2 * i + 1]);
        i += 1;
    }
    out
}

/// # Result of a known-nonce test, per chip in chain order
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct KnownNonceReport {
    /// Headers whose nonce belongs to each chip.
    pub expected: Vec<u16, MAX_ASIC_CNT>,
    /// Of those, the ones the chip returned in time.
    pub found: Vec<u16, MAX_ASIC_CNT>,
    /// Headers whose nonce belongs to no chip of the chain.
    pub skipped: u16,
}

impl KnownNonceReport {
    /// ## Chips that missed at least one of their nonces
    pub fn failing_chips(&self) -> impl Iterator<Item = usize> + '_ {
        self.expected
            .iter()
            .zip(&self.found)
            .enumerate()
            .filter(|(_, (e, f))| f < e)
            .map(|(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn built_in_headers_hash_below_their_target() {
        for block in BITCOIN_BLOCKS_0_TO_2 {
            let mut header = [0u8; 80];
            header[0..4].copy_from_slice(&block.version.to_le_bytes());
            header[4..36].copy_from_slice(&block.prev_block_header_hash);
            header[4..36].reverse();
            header[36..68].copy_from_slice(&block.merkle_root);
            header[36..68].reverse();
            header[68..72].copy_from_slice(&block.n_time.to_le_bytes());
            header[72..76].copy_from_slice(&block.n_bits.to_le_bytes());
            header[76..80].copy_from_slice(&block.nonce.to_le_bytes());
            let hash = Sha256::digest(Sha256::digest(header));
            // difficulty 1: the 4 most significant bytes are zero
            assert_eq!(hash[28..], [0, 0, 0, 0]);
        }
    }

    #[test]
    fn each_block_is_the_previous_one_of_the_next() {
        let mut header = [0u8; 80];
        let block = BITCOIN_BLOCKS_0_TO_2[0];
        header[0..4].copy_from_slice(&block.version.to_le_bytes());
        header[36..68].copy_from_slice(&block.merkle_root);
        header[36..68].reverse();
        header[68..72].copy_from_slice(&block.n_time.to_le_bytes());
        header[72..76].copy_from_slice(&block.n_bits.to_le_bytes());
        header[76..80].copy_from_slice(&block.nonce.to_le_bytes());
        let mut hash: [u8; 32] = Sha256::digest(Sha256::digest(header)).into();
        hash.reverse();
        assert_eq!(hash, BITCOIN_BLOCKS_0_TO_2[1].prev_block_header_hash);
    }
}
