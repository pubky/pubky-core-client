mod transport;
mod crypto;

use transport::Resolver;
use crypto::DeterministicKeyPair;
use transport::http;

/// Create a new account at the config homeserver
pub fn signup(seed: &str) -> Result<&str, Error> {
    // TODO:
    // generate keypair from seed
    let keypair = DeterministicKeyPair::generate(Some(seed));
    // send user root signature as signup
    // create signed packet with keypair and homeserverId
    // userId = encode public key into z32
    // PUT signed packet to homeserver /mvp/users/{userId}/pkarr as `application/octet-stream`
    // cache homeserverUrl accessible via userId
    // zeroize private keypair
    // return userId

}

/// Login to an account at the config homeserver
pub fn login(seed: &str) -> Result<&str, Error> {
    /// TOOD:
    // create keypair from seed
    let keypair = DeterministicKeyPair::generate(Some(seed));
    // send user root signature as login
    // zeroize private keypair
    // return null or userId ?
}

/// Logout from a specific account at the config homeserver
pub fn logout(userId: &str) -> Result<&str, Error> {
    // TODO:
    // DELETE /mvp/session/{userId}
}

/// Examine the current session at the config homeserver
pub fn session() -> Result<&str, Error> {
    // TODO:
    // GET /mvp/session
    // return response
}


/// Get challenge, sign it and authenticate
pub fn send_user_root_signature(sig_type: &str, keypair: &str) -> Result<&str, Error> {
    // TODO:
    // get challenge
    // sign challenge
    // encode userId to z32
    // depending type "signup" or "login" send challenge signature to homeserver to 
    //   - mvep/users/{userId} - for signup
    //   - mvep/session/{userId} - for Login
    // return userId
}

/// Get challenge
pub fn get_challenge(&self) -> Result<&str, Error> {
    // TODO:
    // GET /mvp/challenge
    // desserialize response into challenge
    // return challenge
}
