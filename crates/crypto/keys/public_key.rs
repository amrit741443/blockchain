use core::fmt;
use std::cmp::Ordering;

use crate::{Hash, error::CryptoError, keys::Signature};
use ed25519_dalek::{Verifier, VerifyingKey as DalekVerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Copy, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublicKey(pub(crate) DalekVerifyingKey);

impl PublicKey {
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, CryptoError> {
        let key =
            DalekVerifyingKey::from_bytes(bytes).map_err(|_| CryptoError::InvalidPublicKeyBytes)?;

        Ok(PublicKey(key))
    }

    pub fn as_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn verify(&self, hash: &Hash, signature: &Signature) -> Result<(), CryptoError> {
        self.0
            .verify(hash.as_bytes(), &signature.0)
            .map_err(|_| CryptoError::VerificationFailed)
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Formats as a hex string (e.g., 0x1234...)
        write!(f, "0x{}", hex::encode(self.0.as_bytes()))
    }
}

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PublicKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.to_bytes().cmp(&other.0.to_bytes())
    }
}
