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
    pub homeserver_url: &mut Option<Url>,
    pub session_id: &mut Option<String>,
}

impl Auth {
    pub fn new(resolver: Resolver, homeserver_url: Option<&Url>) -> Auth {
        Auth {
            resolver,
            session_id: None,
            homeserver_url,
        }
    }

    /// Create a new account at the config homeserver
    pub fn signup(seed: &str, relay_url: Option<&Url>) -> Result<&str, String> {
        let key_pair = DeterministicKeyGen::generate(Some(seed));
        let user_id = match self.send_user_root_signature(SigType::Signup, key_pair, &relay_url) {
            Ok(user_id) => user_id,
            Err(e) => return Err(format!("Error signing up: {}", e)),
        };

        if &self.homeserver_url.is_none() {
            &self.homeserver_url = match &self
                .resolver
                .resolve_homeserver(&key_pair.public_key, &relay_url)
            {
                Ok(url) => Some(url),
                Err(e) => return Err(fromat!("Error resolving homeserver: {}", e)),
            };
        }

        let target_url = match relay_url {
            Some(url) => url.join(format!("/mvp/users/{}/pkarr", user_id)).unwrap(),
            None => &self
                .homeserver_url
                .unwrap()
                .join(format!("/mvp/users/{}/pkarr", user_id))
                .unwrap(),
        };

        let _ = match &self
            .resolver
            .publish(&key_pair, &self.homeserver_url.unwrap(), target_url)
        {
            Ok(_) => (),
            Err(e) => return Err(format!("Error publishing public key: {}", e)),
        };

        crypto::zeroize(&key_pair.private_key.as_mut());

        return Ok(user_id);
    }

    /// Login to an account at the config homeserver
    pub fn login(&self, seed: &str, relay_url: Option<&Url>) -> Result<&str, Error> {
        let key_pair = DeterministicKeyGen::generate(Some(seed));
        let user_id = match self.send_user_root_signature(SigType::Login, key_pair, &relay_url) {
            Ok(user_id) => user_id,
            Err(e) => return Err(format!("Error signing up: {}", e)),
        };

        crypto::zeroize(&key_pair.private_key.as_mut());

        return Ok(user_id);
    }

    /// Logout from a specific account at the config homeserver
    pub fn logout(user_id: &str) -> Result<(), String> {
        if &self.session_id.is_none() {
            return Err("No session found".to_string());
        }

        if &self.homeserver_url.is_none() {
            return Err("No homeserver found".to_string());
        }

        match request(
            Method::DELETE,
            &self
                .homeserver_url
                .unwrap()
                .join(format!("/mvp/session/{}", user_id))
                .unwrap(),
            &mut self.session_id,
            None,
            None,
        ) {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("Error logging out: {}", e)),
        }
    }

    /// Examine the current session at the config homeserver
    pub fn session(&self) -> Result<&str, String> {
        // GET /mvp/session
        if &self.homeserver_url.is_none() {
            return Err("No homeserver found".to_string());
        }

        let url = &self.homeserver_url.unwrap().join("/mvp/session").unwrap();

        match request(Method::GET, url, &mut self.session_id, None, None) {
            Ok(response) => {
                // TODO: proper format of response
                // {
                //  users: {
                //    [userId: string]: {
                //      permissions: Array<any>
                //    }
                //  }
                // } | null
                // let session = serde_json::from_str(response).unwrap();
                Ok(response)
            }
            Err(e) => return Err(format!("Error getting session: {}", e)),
        }
    }

    /// Get challenge, sign it and authenticate
    fn send_user_root_signature(
        &self,
        sig_type: &SigType,
        key_pair: Keypair,
        relay_url: Option<&Url>,
    ) -> Result<&str, String> {
        let path = match sig_type {
            SigType::Signup => format!("/mvp/users/{}/pkarr", user_id),
            SigType::Login => format!("/mvp/session/{}", user_id),
            _ => return Err("Invalid signature type"),
        };

        let challenge = self.get_challenge(key_pair.public_key, None);
        let signature = key_pair.sign(&challenge.signable);
        if signature.as_str().len() != 64 {
            return Err("Invalid signature length");
        }
        let user_id = key_pair.to_z32();

        if &self.homeserver_url.is_none() {
            &self.homeserver_url = match &self
                .resolver
                .resolve_homeserver(&key_pair.public_key, &relay_url)
            {
                Ok(url) => Some(url),
                Err(e) => return Err(fromat!("Error resolving homeserver: {}", e)),
            };
        }

        let url = &self.homeserver_url.unwrap().join(path).unwrap();

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
            Ok(_) => Ok(user_id),
            Err(e) => return Err(format!("Error sending user root signature: {}", e)),
        }
    }

    /// Get challenge
    fn get_challenge(
        &self,
        public_key: &PublicKey,
        relay_url: Option<&Url>,
    ) -> Result<&Challenge, Error> {
        if &self.homeserver_url.is_none() {
            &self.homeserver_url = match &self.resolver.resolve_homeserver(&public_key, &relay_url)
            {
                Ok(url) => Some(url),
                Err(e) => return Err(fromat!("Error resolving homeserver: {}", e)),
            };
        };

        let url = &self.homeserver_url.unwrap().join("/mvp/challenge").unwrap();

        match request(Method::GET, url, &mut self.session_id, Some(&headers), None) {
            Ok(response) => Challenge::deserialize(response),
            Err(e) => return Err(format!("Error getting challenge: {}", e)),
        }
    }
}
