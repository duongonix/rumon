//! Binary metadata diffing.

/// Binary file metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BinaryMetadata {
    /// File size in bytes.
    pub size: u64,
    /// Content hash.
    pub hash: u64,
}

/// Binary diff summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BinaryDiff {
    /// Previous metadata.
    pub before: BinaryMetadata,
    /// Current metadata.
    pub after: BinaryMetadata,
    /// Whether the size changed.
    pub size_changed: bool,
    /// Whether the hash changed.
    pub hash_changed: bool,
}

/// Computes binary metadata from bytes.
#[must_use]
pub fn metadata(bytes: &[u8]) -> BinaryMetadata {
    BinaryMetadata {
        size: bytes.len() as u64,
        hash: fnv1a(bytes),
    }
}

/// Computes a binary diff from two byte slices.
#[must_use]
pub fn diff_binary(before: &[u8], after: &[u8]) -> BinaryDiff {
    let before = metadata(before);
    let after = metadata(after);
    BinaryDiff {
        before,
        after,
        size_changed: before.size != after.size,
        hash_changed: before.hash != after.hash,
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    bytes.iter().fold(OFFSET, |hash, byte| {
        let hash = hash ^ u64::from(*byte);
        hash.wrapping_mul(PRIME)
    })
}

#[cfg(test)]
mod tests {
    use super::diff_binary;

    #[test]
    fn detects_binary_hash_change() {
        let diff = diff_binary(b"abc", b"abd");

        assert!(!diff.size_changed);
        assert!(diff.hash_changed);
    }
}
