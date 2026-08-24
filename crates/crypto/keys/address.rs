use core::fmt;

use crate::PublicKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Address(pub PublicKey);

impl Address {
    pub fn new(public_key: PublicKey) -> Self {
        Self(public_key)
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.0
    }
}

impl From<PublicKey> for Address {
    fn from(public_key: PublicKey) -> Self {
        Address(public_key)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Formats as a hex string (e.g., 0x1234...)
        write!(f, "0x{}", hex::encode(self.0.as_bytes()))
    }
}
