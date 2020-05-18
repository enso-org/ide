//! Utilities related to UUID: extensions to the `uuid::Uuid`, the binary protocol's `EnsoUUID`
//! and conversions between them.

use crate::prelude::*;

use crate::generated::binary_protocol_generated::org::enso::languageserver::protocol::binary::EnsoUUID;

impl EnsoUUID {
    /// Creates a new random EnsoUUID.
    pub fn new_v4() -> EnsoUUID {
        Uuid::new_v4().into()
    }
}

pub trait UuidExt {
    /// The most significant 64 bits of this UUID's 128 bit value.
    ///
    /// Compatible with `java.util.UUID.getMostSignificantBits()`.
    fn most_significant_bits(&self) -> i64;

    /// The least significant 64 bits of this UUID's 128 bit value.
    ///
    /// Compatible with `java.util.UUID.getLeastSignificantBits()`.
    fn least_significant_bits(&self) -> i64;

    /// Constructs a new UUID using the specified data.
    ///
    /// `most_significant_bits` is used for the most significant 64 bits of the UUID and
    /// `least_significant_bits` becomes the least significant 64 bits of the UUID.
    fn from_i64_pair(least_significant_bits:i64, most_significant_bits:i64) -> Self;
}

impl UuidExt for Uuid {
    fn most_significant_bits(&self) -> i64 {
        i64::from_be_bytes(self.as_bytes()[..8].try_into().unwrap())
    }

    fn least_significant_bits(&self) -> i64 {
        i64::from_be_bytes(self.as_bytes()[8..].try_into().unwrap())
    }

    fn from_i64_pair(least_significant_bits:i64, most_significant_bits:i64) -> Self {
        let most_significant_bytes  = most_significant_bits.to_le_bytes();
        let least_significant_bytes = least_significant_bits.to_le_bytes();
        let all_bytes = least_significant_bytes.iter().chain(most_significant_bytes.iter()).rev();

        let mut bytes : [u8;16] = [default();16];
        for (dst,src) in bytes.iter_mut().zip(all_bytes) {
            *dst = *src;
        }

        Uuid::from_bytes(bytes)
    }
}

impls! { From + &From <Uuid> for EnsoUUID {
    |uuid|
        EnsoUUID::new(uuid.least_significant_bits() as u64, uuid.most_significant_bits() as u64)
}}

impls! { From + &From <EnsoUUID> for Uuid {
    |enso_uuid| {
        let most_significant_bytes  = enso_uuid.mostSigBits().to_le_bytes();
        let least_significant_bytes = enso_uuid.leastSigBits().to_le_bytes();
        let all_bytes = least_significant_bytes.iter().chain(most_significant_bytes.iter()).rev();

        let mut bytes : [u8;16] = [default();16];
        for (dst,src) in bytes.iter_mut().zip(all_bytes) {
            *dst = *src;
        }

        Uuid::from_bytes(bytes)
    }
}}

#[cfg(test)]
mod tests {
    use super::*;
    //use std::f32::consts::PI;

    #[test]
    fn uuid_round_trips() {
        //let uuid = Uuid::new_v4();
        let uuid = Uuid::parse_str("6de39f7b-df3a-4a3c-84eb-5eaf96ddbac2").unwrap();
        let enso = EnsoUUID::from(uuid);
        let uuid2 = Uuid::from(enso);
        assert_eq!(uuid,uuid2);
    }
}

