use crate::transport::crypto;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get current time in seconds
///
/// # Examples
/// ```
/// use pubky_core_client::utils;
/// let now = utils::now();
///
/// let then = utils::now();
///
/// assert!(now <= then);
/// ```
pub fn now() -> u64 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// Generate 32bytes long random seed
/// # Examples
/// ```
/// use pubky_core_client::utils;
/// let seed_1 = utils::generate_seed();
/// let seed_2 = utils::generate_seed();
/// assert_ne!(seed_1, seed_2);
/// ```
pub fn generate_seed() -> [u8; 32] {
    crypto::random_bytes(32).try_into().unwrap()
}

/// Generate a new ED25519 key pair
///
/// # Examples
/// ```
/// use pubky_core_client::utils;
/// let new_keypair = utils::generate_keypair(None);
///
/// let seed = b"it is a seed for key generation!";
/// let keypair_1_from_seed = utils::generate_keypair(Some(seed));
/// let keypair_2_from_seed = utils::generate_keypair(Some(seed));
///
/// assert_ne!(new_keypair.secret_key(), keypair_1_from_seed.secret_key());
/// assert_ne!(new_keypair.public_key(), keypair_1_from_seed.public_key());
/// assert_eq!(keypair_1_from_seed.secret_key(), keypair_2_from_seed.secret_key());
/// assert_eq!(keypair_1_from_seed.public_key(), keypair_2_from_seed.public_key());
/// ```
pub fn generate_keypair(seed: Option<&[u8; 32]>) -> crypto::Keypair {
    crypto::DeterministicKeyGen::generate(seed)
}

/// Get user id (z32 encoded public key)
///
/// # Examples
/// ```
/// use pubky_core_client::utils;
/// let user_id = utils::get_user_id(None);
///
/// let seed = b"it is a seed for key generation!";
/// let user_id_1_from_seed = utils::get_user_id(Some(seed));
/// let user_id_2_from_seed = utils::get_user_id(Some(seed));
///
/// assert_ne!(user_id_1_from_seed, user_id);
/// assert_eq!(user_id_1_from_seed, user_id_2_from_seed);
/// ```
pub fn get_user_id(seed: Option<&[u8; 32]>) -> String {
    let keypair = generate_keypair(seed);
    keypair.to_z32()
}
