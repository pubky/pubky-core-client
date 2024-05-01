use crate::transport::crypto;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> u64 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// Generate a new key pair
pub fn generate_keypair(seed: Option<&[u8; 32]>) -> crypto::Keypair {
    crypto::DeterministicKeyGen::generate(seed)
}

/// Get user id
pub fn get_user_id(seed: Option<&[u8; 32]>) -> String {
    let keypair = generate_keypair(seed);
    keypair.to_z32()
}
