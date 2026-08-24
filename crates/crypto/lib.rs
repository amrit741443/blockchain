mod error;
mod keys;
mod primitives;

pub use error::*;
pub use keys::*;
pub use primitives::Hash;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_hash_generation_and_conversions() {
        let data = b"blockchain_transaction";
        let hash1 = Hash::digest(data);
        let hash2 = Hash::digest(data);

        // Hashes of identical data must match
        assert_eq!(hash1, hash2);

        let raw_bytes: [u8; 32] = hash1.to_bytes();
        let reconstructed_hash = Hash::from_bytes(raw_bytes);
        assert_eq!(hash1, reconstructed_hash);

        // Test AsRef implementation
        // assert!(format!("{}", hash1).starts_with("0x"));
    }

    #[test]
    fn test_keypair_generation_and_public_key_derivation() {
        let keypair = Keypair::generate();
        let public_key = keypair.public_key();

        //Round-trip test: Sign a hash and verify the signature using the derived public key
        let pk_bytes = public_key.as_bytes();
        let reconstructed_public_key = PublicKey::from_bytes(&pk_bytes).unwrap();
        assert_eq!(public_key.as_bytes(), reconstructed_public_key.as_bytes());

        assert_eq!(public_key, reconstructed_public_key);
    }

    #[test]
    fn test_valid_signature_signing_and_verification() {
        let keypair = Keypair::generate();
        let public_key = keypair.public_key();

        let message_hash = Hash::digest(b"Alice pays Bob 50 tokens");
        let signature = keypair.sign(&message_hash);

        // Verify the signature
        assert!(public_key.verify(&message_hash, &signature).is_ok());
    }

    #[test]
    fn test_signature_fails_on_tampered_message() {
        let keypair = Keypair::generate();
        let public_key = keypair.public_key();

        let original_hash = Hash::digest(b"Alice pays Bob 50 tokens");
        let signature = keypair.sign(&original_hash);

        // Tamper with the message
        let tampered_hash = Hash::digest(b"Alice pays Bob 5000 tokens");

        // Verify the signature against the tampered message
        let result = public_key.verify(&tampered_hash, &signature);
        assert_eq!(result, Err(CryptoError::VerificationFailed));
    }

    #[test]
    fn test_signature_fails_with_wrong_public_key() {
        let alice_keypair = Keypair::generate();
        let bob_keypair = Keypair::generate();

        let message_hash = Hash::digest(b"Transfer 10 tokens");
        let alice_signature = alice_keypair.sign(&message_hash);

        // Verifying Alice's signature using Bob's public key must fail
        let result = bob_keypair
            .public_key()
            .verify(&message_hash, &alice_signature);
        assert_eq!(result, Err(CryptoError::VerificationFailed));
    }

    #[test]
    fn test_signature_byte_serialization_roundtrip() {
        let keypair = Keypair::generate();
        let message_hash = Hash::digest(b"Test message");
        let signature = keypair.sign(&message_hash);

        //Convert signature to bytes and back
        let sig_bytes: [u8; 64] = signature.to_bytes();
        let reconstructed_sig = Signature::from_bytes(&sig_bytes);

        assert_eq!(signature, reconstructed_sig);
    }

    #[test]
    fn test_keypair_seed_recovery() {
        let original_keypair = Keypair::generate();
        let seed = original_keypair.to_seed();

        //Restore keypair from seed
        let recovered_keypair = Keypair::from_seed(&seed);

        assert_eq!(
            original_keypair.public_key(),
            recovered_keypair.public_key()
        );
    }
}
