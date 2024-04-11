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
        let key_pair: &Keypair = &DeterministicKeyGen::generate(Some(seed));
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

        // XXX: this seems wrong, jsut home server url should be enough
        let target_url = self
            .homeserver_url
            .clone()
            .unwrap()
            .join(&format!("/mvp/users/{}/pkarr", user_id).as_str())
            .unwrap();

        let url = self.homeserver_url.clone().unwrap();

        // XXX
        // let _ = match &self.resolver.publish(key_pair, &url, Some(&target_url)) {
        let _ = match &self.resolver.publish(key_pair, &url, None) {
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
    pub fn logout(&mut self, user_id: &str) -> Result<String, String> {
        if self.session_id.is_none() {
            return Err("No session found".to_string());
        }

        if self.homeserver_url.is_none() {
            return Err("No homeserver found".to_string());
        }

        let url = self
            .homeserver_url
            .clone()
            .unwrap()
            .join(&format!("/mvp/session/{}", user_id).as_str())
            .unwrap();

        match request(Method::DELETE, url, &mut self.session_id, None, None) {
            Ok(_) => Ok(self.session_id.take().unwrap()),
            Err(e) => return Err(format!("Error logging out: {}", e)),
        }
    }

    /// Examine the current session at the config homeserver
    pub fn session(&mut self) -> Result<String, String> {
        if self.homeserver_url.is_none() {
            return Err("No homeserver found".to_string());
        }

        let url = self
            .homeserver_url
            .clone()
            .unwrap()
            .join("/mvp/session")
            .unwrap();

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
        if signature.len() != 128 {
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

        let url = self
            .homeserver_url
            .clone()
            .unwrap()
            .join(path.as_str())
            .unwrap();

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

        let url = self
            .homeserver_url
            .clone()
            .unwrap()
            .join("/mvp/challenge")
            .unwrap();

        match request(Method::GET, url.clone(), &mut self.session_id, None, None) {
            Ok(response) => Ok(Challenge::deserialize(response.as_bytes())),
            Err(e) => return Err(format!("Error getting challenge: {}", e)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::{setup_datastore, HttpMockParams};
    use crate::transport::crypto;
    use mainline::dht::Testnet;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn now() -> u64 {
        let now = SystemTime::now();
        now.duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    #[test]
    fn auth_walk_through() {
        let testnet = Testnet::new(10);

        let seed = b"it is a seed for key generation!";
        let key_pair: Keypair = DeterministicKeyGen::generate(Some(seed));
        let user_id = key_pair.to_z32();

        let challenge = Challenge::create(now() + 1000, None);

        let get_challange_mock_params = HttpMockParams {
            method: &Method::GET,
            path: "/mvp/challenge",
            body: &challenge.serialize(),
            status: 200,
            headers: vec![],
        };

        let path = format!("/mvp/users/{}/pkarr", user_id);
        let send_user_root_signature_signup_mock_params = HttpMockParams {
            method: &Method::PUT,
            path: path.as_str(),
            headers: vec![("Set-Cookie", "sessionId=123")],
            status: 200,
            body: &b"ok".to_vec(),
        };

        let path = format!("/mvp/session/{}", user_id);
        let send_user_root_signature_login_mock_params = HttpMockParams {
            method: &Method::PUT,
            path: path.as_str(),
            headers: vec![("Set-Cookie", "sessionId=1234")],
            status: 200,
            body: &b"ok".to_vec(),
        };

        let get_session_mock_params = HttpMockParams {
            method: &Method::GET,
            path: "/mvp/session",
            headers: vec![("Set-Cookie", "sessionId=12345")],
            body: &b"session".to_vec(), // TODO: proper session object
            status: 200,
        };

        let path = format!("/mvp/session/{}", user_id);
        let logout_mock_params = HttpMockParams {
            method: &Method::DELETE,
            path: path.as_str(),
            status: 200,
            body: &b"ok".to_vec(),
            headers: vec![],
        };

        let server = setup_datastore(vec![
            get_challange_mock_params,
            send_user_root_signature_signup_mock_params,
            send_user_root_signature_login_mock_params,
            get_session_mock_params,
            logout_mock_params,
        ]);

        let mut resolver = Resolver::new(None, Some(&testnet.bootstrap));
        let _ = resolver.publish(&key_pair, &Url::parse(&server.url()).unwrap(), None).unwrap();

        let mut auth = Auth::new(resolver, None);

        // TEST SIGNUP
        let user_id = auth.signup(seed, None).unwrap();
        assert_eq!(user_id, user_id);
        assert_eq!(auth.homeserver_url, Some(Url::parse(&server.url()).unwrap()));
        assert_eq!(auth.session_id, Some("123".to_string()));

        // TEST LOGOUT
        let session_id = auth.logout(&user_id).unwrap();
        assert_eq!(auth.session_id, None);
        assert_eq!(session_id, "123");

        // // TEST LOGIN
        let user_id = auth.login(seed, None).unwrap();
        assert_eq!(user_id, user_id);
        assert_eq!(auth.homeserver_url, Some(Url::parse(&server.url()).unwrap()));
        assert_eq!(auth.session_id, Some("1234".to_string()));

        // TEST SESSION
        let session = auth.session().unwrap();
        assert_eq!(session, "session".to_string());
        assert_eq!(auth.session_id, Some("12345".to_string()));
        assert_eq!(auth.homeserver_url, Some(Url::parse(&server.url()).unwrap()));
    }
}
