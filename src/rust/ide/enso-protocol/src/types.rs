//! Common types of JSON-RPC-based Enso services used by both Project Manager and Language Server.

use crate::prelude::*;

use serde::Serialize;
use serde::Deserialize;

/// Time in UTC time zone.
pub type UTCDateTime = chrono::DateTime<chrono::FixedOffset>;

/// SHA3-224 hash digest.
#[derive(Hash,Debug,Clone,PartialEq,Eq,Serialize,Deserialize,Shrinkwrap)]
pub struct Sha3_224(String);

impl Sha3_224 {
    /// Create new SHA3-224 digest from `data`.
    pub fn new(data:&[u8]) -> Self {
        use sha3::Digest;
        let mut hasher = sha3::Sha3_224::new();
        hasher.input(data);
        let result = hasher.result();
        let digest = hex::encode(result[..].to_vec());
        Self(digest)
    }
}

// // =============
// // === Tests ===
// // =============
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn sha3_224() {
//         let digest   = Sha3_224::new(b"abc");
//         let expected = [230,66,130,76,63,140,242,74,208,146,52,238,125,60,118,111,201,163,165,22,
//                         141,12,148,173,115,180,111,223];
//         assert_eq!(digest.data()[..],expected);
//     }
// }
