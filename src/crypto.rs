// Re-exporting the blake3 crate
#[allow(unused_imports)]
pub use blake3;

use rand::rngs::OsRng;
use rand::RngCore;

// Re-exporting public key for the challenge
#[allow(unused_imports)]
pub use pkarr::PublicKey;

use ed25519_dalek::SigningKey;

use pkarr::Keypair;

pub trait DeterministicKeyGen {
    fn generate(seed: Option<&[u8; 32]>) -> Self;
}

impl DeterministicKeyGen for Keypair {
    fn generate(seed: Option<&[u8; 32]>) -> Self {
        match seed {
            Some(seed) => {
                let signing_key = SigningKey::from_bytes(seed);
                Keypair::from_secret_key(&signing_key.to_bytes())
            }
            None => Keypair::random(),
        }
    }
}

pub fn zeroize(buf: &mut [u8]) {
    for byte in buf.iter_mut() {
        *byte = 0;
    }
}

pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut rng = OsRng;
    let mut buffer = vec![0u8; len];
    rng.fill_bytes(&mut buffer);

    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeroize() {
        let mut buf = vec![1, 2, 3, 4, 5];
        zeroize(&mut buf);
        assert_eq!(buf, vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_deterministic_keygen() {
        let seed_1 = b"it is a seed for key generation!";
        let seed_2 = b"not a seed for a key generation!";

        assert_eq!(
            Keypair::generate(Some(seed_1)).to_z32(),
            Keypair::generate(Some(seed_1)).to_z32()
        );

        assert_ne!(
            Keypair::generate(Some(seed_1)).to_z32(),
            Keypair::generate(Some(seed_2)).to_z32()
        );
    }

    #[test]
    fn test_random_bytes() {
        let len = 32;
        let bytes_1 = random_bytes(len);
        let bytes_2 = random_bytes(len);

        assert_eq!(bytes_1.len(), len);
        assert_eq!(bytes_2.len(), len);
        assert_ne!(bytes_1, bytes_2);
    }
}
