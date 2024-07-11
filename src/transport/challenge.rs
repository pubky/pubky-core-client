use crate::error::ChallengeError as Error;
use crate::transport::crypto;
use crate::utils::now;

#[derive(Debug)]
pub struct Challenge {
    pub value: [u8; 32],
    pub expires_at: u64,
    pub signable: [u8; 32],
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

    // Will either be removed or used after update of auth
    #[allow(dead_code)]
    pub fn create(expires_at: u64, challenge: Option<[u8; 32]>) -> Self {
        // Lazily generate a challenge if none is provided
        let challenge = challenge.unwrap_or_else(|| {
            crypto::random_bytes(32)
                .try_into()
                .expect("Failed to generate challenge")
        });
        let signable = Self::signable(&challenge);

        Self::new(challenge, expires_at, signable)
    }

    // Will either be removed or used after update of auth
    #[allow(dead_code)]
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

    // Will either be removed or used after update of auth
    #[allow(dead_code)]
    pub fn expired(&self) -> bool {
        self.expires_at <= now()
    }

    pub fn signable(challenge: &[u8]) -> [u8; 32] {
        crypto::blake3::derive_key(CONTEXT, challenge)
    }

    // Will either be removed or used after update of auth
    #[allow(dead_code)]
    pub fn verify(
        &self,
        signature: &crypto::Signature,
        public_key: &crypto::PublicKey,
    ) -> Result<(), Error> {
        if self.expired() {
            return Err(Error::Expired);
        }

        match public_key.verify(&self.signable, signature) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::InvalidSignature),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge() {
        let challenge = Challenge::create(now(), None);
        let serialized = challenge.serialize();
        let deserialized = Challenge::deserialize(&serialized);

        assert_eq!(challenge.value, deserialized.value);
        assert_eq!(challenge.expires_at, deserialized.expires_at);
        assert!(challenge.expires_at <= now())
    }

    #[test]
    fn test_signable() {
        let challenge = crypto::random_bytes(32);
        let signable = Challenge::signable(&challenge);

        assert_eq!(signable.len(), 32);
    }

    #[test]
    fn test_expired() {
        let challenge = Challenge::create(now() - 1000, None);

        assert!(challenge.expired());
    }

    #[test]
    fn test_verify() {
        let challenge = Challenge::create(now() + 1000, None);
        let keypair = pkarr::Keypair::random();
        let signature = keypair.sign(&challenge.signable);

        assert!(challenge.verify(&signature, &keypair.public_key()).is_ok());
    }
}
