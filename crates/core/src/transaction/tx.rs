use crate::error::TransactionError;

use crypto::{Hash, Keypair, PublicKey, Signature};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    //TODO: pub is used for only testing
    sender: PublicKey,
    receiver: PublicKey,
    amount: u64,
    nonce: u64,
    signature: Signature,
    fee: u64,
}

impl Transaction {
    pub fn new(
        sender_keypair: &Keypair,
        receiver: PublicKey,
        amount: u64,
        nonce: u64,
        fee: u64,
    ) -> Result<Self, TransactionError> {
        let sender = sender_keypair.public_key();

        Self::validate_fields(&sender, &receiver, amount)?;

        let payload = Self::signing_payload(&sender, &receiver, amount, nonce, fee);

        let bytes = bincode::serialize(&payload).map_err(TransactionError::Serialization)?;

        let signing_hash = Hash::digest(&bytes);

        let signature = sender_keypair.sign(&signing_hash);

        Ok(Self {
            sender,
            receiver,
            amount,
            nonce,
            fee,
            signature,
        })
    }

    fn signing_payload(
        sender: &PublicKey,
        receiver: &PublicKey,
        amount: u64,
        nonce: u64,
        fee: u64,
    ) -> ([u8; 32], [u8; 32], u64, u64, u64) {
        (sender.as_bytes(), receiver.as_bytes(), amount, nonce, fee)
    }

    fn validate_fields(
        sender: &PublicKey,
        receiver: &PublicKey,
        amount: u64,
    ) -> Result<(), TransactionError> {
        if amount == 0 {
            return Err(TransactionError::ZeroAmount);
        }

        if sender == receiver {
            return Err(TransactionError::SelfTransfer);
        }

        Ok(())
    }

    pub fn signing_hash(&self) -> Result<Hash, TransactionError> {
        let payload = Self::signing_payload(
            &self.sender,
            &self.receiver,
            self.amount,
            self.nonce,
            self.fee,
        );

        let bytes = bincode::serialize(&payload).map_err(TransactionError::Serialization)?;

        Ok(Hash::digest(&bytes))
    }

    pub fn verify(&self) -> Result<(), TransactionError> {
        Self::validate_fields(&self.sender, &self.receiver, self.amount)?;

        let signing_hash = self.signing_hash()?;

        self.sender
            .verify(&signing_hash, &self.signature)
            .map_err(TransactionError::InvalidSignature)?;

        Ok(())
    }

    pub fn tx_id(&self) -> Result<Hash, TransactionError> {
        let payload = self.id_payload();

        let bytes = bincode::serialize(&payload).map_err(TransactionError::Serialization)?;

        Ok(Hash::digest(&bytes))
    }

    fn id_payload(&self) -> ([u8; 32], [u8; 32], u64, u64, u64, &Signature) {
        (
            self.sender.as_bytes(),
            self.receiver.as_bytes(),
            self.amount,
            self.nonce,
            self.fee,
            &self.signature,
        )
    }

    pub fn sender(&self) -> &PublicKey {
        &self.sender
    }

    pub fn receiver(&self) -> &PublicKey {
        &self.receiver
    }

    pub fn amount(&self) -> u64 {
        self.amount
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn fee(&self) -> u64 {
        self.fee
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}
