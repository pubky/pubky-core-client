use crate::crypto;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Challenge {
    value: [u8; 32],
    expires_at: u64,
    signable: [u8; 32],
}

static CONTEXT: &str = "pubky:homeserver:challenge";

impl Challenge {
    pub fn new(value: [u8; 32], expires_at: u64, signable: [u8; 32]) -> Self {
        Self {
            value,
            expires_at,
            signable,
        }
    }

    pub fn create(expires_at: u64, challenge: Option<[u8; 32]>) -> Self {
        let challenge = match challenge {
            Some(challenge) => challenge,
            None => crypto::random_bytes(32).try_into().expect("Something went wrong")
        };
        let signable = Self::signable(&challenge);

        Self::new(challenge, expires_at, signable)
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(40);
        bytes.extend_from_slice(&self.value);
        bytes.extend_from_slice(&self.expires_at.to_be_bytes());

        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let mut value = [0; 32];
        value.copy_from_slice(&bytes[0..32]);

        let mut expires_at = [0; 8];
        expires_at.copy_from_slice(&bytes[32..40]);
        let expires_at = u64::from_be_bytes(expires_at);

        Self::new(value, expires_at, Self::signable(&value))
    }

    pub fn expired(&self) -> bool {
        self.expires_at <= Self::now()
    }

    pub fn signable(challenge: &[u8]) -> [u8; 32] {
        crypto::blake3::derive_key(CONTEXT, challenge)
    }

    pub fn verify(
        &self,
        signature: &crypto::Signature,
        public_key: &crypto::PublicKey,
    ) -> Result<(), &'static str> {
        if self.expired() {
            return Err("Expired challenge");
        }

        let _foo = public_key.verify(&self.signable, signature);

        Ok(())
    }

    pub fn now() -> u64 {
        let now = SystemTime::now();
        now.duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge() {
        let challenge = Challenge::create(Challenge::now(), None);
        let serialized = challenge.serialize();
        let deserialized = Challenge::deserialize(&serialized);

        assert_eq!(challenge.value, deserialized.value);
        assert_eq!(challenge.expires_at, deserialized.expires_at);
        assert!(challenge.expired())
    }

    #[test]
    fn test_signable() {
        let challenge = crypto::random_bytes(32);
        let signable = Challenge::signable(&challenge);

        assert_eq!(signable.len(), 32);
    }

    #[test]
    fn test_verify() {
        let challenge = Challenge::create(Challenge::now() + 1000, None);
        let keypair = pkarr::Keypair::random();
        let signature = keypair.sign(&challenge.signable);

        assert!(challenge
            .verify(&signature, &keypair.public_key())
            .is_ok());
    }
}
