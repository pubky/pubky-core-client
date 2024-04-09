mod auth;
mod crypto;
mod transport;

use auth::Challenge;
use crypto::{DeterministicKeyGen, Kaypair, PublicKey};
use transport::http::{request, Method};
use transport::Resolver;

pub enum SigType {
    Signup,
    Login,
}

pub struct Auth {
    resolver: Resolver,
    pub session_id: &mut Option<String>,
}

impl Auth {
    pub fn new(resolver: Resolver) -> Auth {
        Auth {
            resolver,
            session_id: None,
        }
    }

    /// Create a new account at the config homeserver
    pub fn signup(seed: &str) -> Result<&str, Error> {
        // TODO:
        // generate keypair from seed
        let keypair = DeterministicKeyGen::generate(Some(seed));
        // send user root signature as signup
        // create signed packet with keypair and homeserverId
        // userId = encode public key into z32
        // PUT signed packet to homeserver /mvp/users/{userId}/pkarr as `application/octet-stream`
        // cache homeserver_url accessible via userId
        // zeroize private keypair
        // return userId
    }

    /// Login to an account at the config homeserver
    pub fn login(seed: &str) -> Result<&str, Error> {
        /// TOOD:
        // create keypair from seed
        let keypair = DeterministicKeyGen::generate(Some(seed));
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
    fn send_user_root_signature(
        &self,
        sig_type: &SigType,
        keypair: Keypair,
        relay_url: Option<&Url>,
    ) -> Result<&str, String> {
        let path = match sig_type {
            SigType::Signup => format!("/mvp/users/{}/pkarr", userId),
            SigType::Login => format!("/mvp/session/{}", userId),
            _ => return Err("Invalid signature type"),
        };

        let challenge = self.get_challenge(keypair.public_key, None);
        let signature = keypair.sign(&challenge.signable);
        if signature.as_str().len() != 64 {
            return Err("Invalid signature length");
        }
        let userId = keypair.to_z32();

        let homeserver_url = match &self.resolver.resolve_homeserver(&public_key, &relay_url) {
            Ok(url) => url,
            Err(e) => return Err(fromat!("Error resolving homeserver: {}", e)),
        };

        let url = homeserver_url.join(path).unwrap();

        let mut headers = HashMap::new();
        headers.insert("Content-Type", "application/octet-stream");
        headers.insert("Content-Length", signature.as_str().len().to_string());

        let response = request(
            Method::PUT,
            url,
            &mut self.session_id,
            Some(&headers),
            Some(&signature),
        );

        match response {
            Ok(_) => Ok(userId),
            Err(e) => return Err(format!("Error sending user root signature: {}", e)),
        }
    }

    /// Get challenge
    fn get_challenge(
        &self,
        public_key: &PublicKey,
        relay_url: Option<&Url>,
    ) -> Result<&Challenge, Error> {
        let homeserver_url = match &self.resolver.resolve_homeserver(&public_key, &relay_url) {
            Ok(url) => url,
            Err(e) => return Err(fromat!("Error resolving homeserver: {}", e)),
        };

        let url = homeserver_url.join("/mvp/challenge").unwrap();

        match request(Method::GET, url, &mut self.session_id, Some(&headers), None) {
            Ok(response) => Challenge::deserialize(response),
            Err(e) => return Err(format!("Error getting challenge: {}", e)),
        }
    }
}
