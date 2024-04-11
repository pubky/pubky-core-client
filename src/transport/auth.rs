use crate::transport::challenge::Challenge;
use crate::transport::crypto::{zeroize, DeterministicKeyGen, Keypair, PublicKey};
use crate::transport::http::{request, HeaderMap, Method, Url};
use crate::transport::resolver::Resolver;

pub enum SigType {
    Signup,
    Login,
}

pub struct Auth<'a> {
    pub homeserver_url: Option<Url>,
    pub session_id: Option<String>,
    resolver: Resolver<'a>,
}

impl Auth<'_> {
    pub fn new(resolver: Resolver, homeserver_url: Option<Url>) -> Auth {
        Auth {
            resolver,
            session_id: None,
            homeserver_url,
        }
    }

    /// Create a new account at the config homeserver
    pub fn signup(
        &mut self,
        seed: &[u8; 32],
        dht_relay_url: Option<&Url>,
    ) -> Result<String, String> {
        let key_pair = &DeterministicKeyGen::generate(Some(seed));
        let user_id = match self.send_user_root_signature(&SigType::Signup, key_pair, dht_relay_url)
        {
            Ok(user_id) => user_id,
            Err(e) => return Err(format!("Error signing up: {}", e)),
        };

        if self.homeserver_url.is_none() {
            self.homeserver_url = match self
                .resolver
                .resolve_homeserver(&key_pair.public_key(), dht_relay_url)
            {
                Ok(url) => Some(url),
                Err(e) => return Err(format!("Error resolving homeserver: {}", e)),
            };
        }

        let target_url = &self
            .homeserver_url
            .clone()
            .unwrap()
            .join(&format!("/mvp/users/{}/pkarr", user_id).as_str())
            .unwrap();

        let url = <Option<Url> as Clone>::clone(&self.homeserver_url)
            .unwrap()
            .clone();
        let _ = match &self.resolver.publish(key_pair, &url, Some(&target_url)) {
            Ok(_) => (),
            Err(e) => return Err(format!("Error publishing public key: {}", e)),
        };

        zeroize(key_pair.secret_key().as_mut());

        return Ok(user_id.to_string());
    }

    /// Login to an account at the config homeserver
    pub fn login(
        &mut self,
        seed: &[u8; 32],
        dht_relay_url: Option<&Url>,
    ) -> Result<String, String> {
        let key_pair = &DeterministicKeyGen::generate(Some(seed));
        let user_id = match self.send_user_root_signature(&SigType::Login, key_pair, dht_relay_url)
        {
            Ok(user_id) => user_id,
            Err(e) => return Err(format!("Error signing up: {}", e)),
        };

        zeroize(key_pair.secret_key().as_mut());

        return Ok(user_id);
    }

    /// Logout from a specific account at the config homeserver
    pub fn logout(&mut self, user_id: &str) -> Result<(), String> {
        if self.session_id.is_none() {
            return Err("No session found".to_string());
        }

        if self.homeserver_url.is_none() {
            return Err("No homeserver found".to_string());
        }

        let url = <Option<Url> as Clone>::clone(&self.homeserver_url)
            .unwrap()
            .clone();
        let url = url
            .join(&format!("/mvp/session/{}", user_id).as_str())
            .unwrap();

        match request(Method::DELETE, url, &mut self.session_id, None, None) {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("Error logging out: {}", e)),
        }
    }

    /// Examine the current session at the config homeserver
    pub fn session(&mut self) -> Result<String, String> {
        // GET /mvp/session
        if self.homeserver_url.is_none() {
            return Err("No homeserver found".to_string());
        }

        let url = <Option<Url> as Clone>::clone(&self.homeserver_url)
            .unwrap()
            .clone();
        let url = url.join("/mvp/session").unwrap();

        match request(Method::GET, url.clone(), &mut self.session_id, None, None) {
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
                Ok(response.to_string())
            }
            Err(e) => return Err(format!("Error getting session: {}", e)),
        }
    }

    /// Get challenge, sign it and authenticate
    fn send_user_root_signature(
        &mut self,
        sig_type: &SigType,
        key_pair: &Keypair,
        dht_relay_url: Option<&Url>,
    ) -> Result<String, String> {
        let challenge = self.get_challenge(&key_pair.public_key(), None);
        let signature = key_pair.sign(&challenge.unwrap().signable).to_string();
        if signature.len() != 64 {
            return Err("Invalid signature length".to_string());
        }
        let user_id = key_pair.to_z32();

        if self.homeserver_url.is_none() {
            self.homeserver_url = match self
                .resolver
                .resolve_homeserver(&key_pair.public_key(), dht_relay_url)
            {
                Ok(url) => Some(url),
                Err(e) => return Err(format!("Error resolving homeserver: {}", e)),
            };
        }

        let path = match sig_type {
            SigType::Signup => format!("/mvp/users/{}/pkarr", user_id),
            SigType::Login => format!("/mvp/session/{}", user_id),
        };

        let url = <Option<Url> as Clone>::clone(&self.homeserver_url)
            .unwrap()
            .clone();
        let url = url.join(path.as_str()).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            "application/octet-stream".try_into().unwrap(),
        );
        headers.insert("Content-Length", signature.len().try_into().unwrap());

        let response = request(
            Method::PUT,
            url.clone(),
            &mut self.session_id,
            Some(&headers),
            Some(signature.to_string()),
        );

        match response {
            Ok(_) => Ok(user_id.to_string()),
            Err(e) => return Err(format!("Error sending user root signature: {}", e)),
        }
    }

    /// Get challenge
    fn get_challenge(
        &mut self,
        public_key: &PublicKey,
        dht_relay_url: Option<&Url>,
    ) -> Result<Challenge, String> {
        if self.homeserver_url.is_none() {
            self.homeserver_url = match self.resolver.resolve_homeserver(&public_key, dht_relay_url)
            {
                Ok(url) => Some(url),
                Err(e) => return Err(format!("Error resolving homeserver: {}", e)),
            };
        };

        let url = <Option<Url> as Clone>::clone(&self.homeserver_url)
            .unwrap()
            .clone();
        let url = url.join("/mvp/challenge").unwrap();

        match request(Method::GET, url.clone(), &mut self.session_id, None, None) {
            Ok(response) => Ok(Challenge::deserialize(response.as_bytes())),
            Err(e) => return Err(format!("Error getting challenge: {}", e)),
        }
    }
}
