use crate::error::AuthError as Error;
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
    ) -> Result<String, Error> {
        let key_pair: &Keypair = &DeterministicKeyGen::generate(Some(seed));
        let user_id = match self.send_user_root_signature(&SigType::Signup, key_pair, dht_relay_url)
        {
            Ok(user_id) => user_id,
            Err(e) => return Err(e),
        };

        if self.homeserver_url.is_none() {
            self.homeserver_url = match self
                .resolver
                .resolve_homeserver(&key_pair.public_key(), dht_relay_url)
            {
                Ok(url) => Some(url),
                Err(e) => return Err(Error::FailedToResolveHomeserver(e)),
            };
        }

        // Re-publish the homeserver url
        let _ = match &self.resolver.publish(
            key_pair,
            &self.homeserver_url.clone().unwrap(),
            dht_relay_url,
        ) {
            Ok(_) => (),
            Err(e) => return Err(Error::FailedToPublishHomeserver(e.clone())),
        };

        zeroize(key_pair.secret_key().as_mut());

        return Ok(user_id.to_string());
    }

    /// Login to an account at the homeserver
    // TODO: add support for login to others homeservers (not part of SDK yet)
    pub fn login(&mut self, seed: &[u8; 32], dht_relay_url: Option<&Url>) -> Result<String, Error> {
        let key_pair = &DeterministicKeyGen::generate(Some(seed));
        let user_id = match self.send_user_root_signature(&SigType::Login, key_pair, dht_relay_url)
        {
            Ok(user_id) => user_id,
            Err(e) => return Err(e),
        };

        zeroize(key_pair.secret_key().as_mut());

        return Ok(user_id);
    }

    /// Logout from a specific account at the config homeserver
    pub fn logout(&mut self, user_id: &str) -> Result<String, Error> {
        if self.homeserver_url.is_none() {
            return Err(Error::NoHomeserver);
        }

        if self.session_id.is_none() {
            return Err(Error::NoSession);
        }

        let url = self
            .homeserver_url
            .clone()
            .unwrap()
            .join(&format!("/mvp/session/{}", user_id).as_str())
            .unwrap();

        match request(Method::DELETE, url, &mut self.session_id, None, None) {
            Ok(_) => Ok(self.session_id.take().unwrap().clone()),
            Err(e) => return Err(Error::FailedToLogout(e)),
        }
    }

    /// Examine the current session at the config homeserver
    pub fn session(&mut self) -> Result<String, Error> {
        if self.homeserver_url.is_none() {
            return Err(Error::NoHomeserver);
        }

        if self.session_id.is_none() {
            return Err(Error::NoSession);
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
            Err(e) => return Err(Error::FailedToRetrieveSession(e)),
        }
    }

    /// Get challenge, sign it and authenticate
    fn send_user_root_signature(
        &mut self,
        sig_type: &SigType,
        key_pair: &Keypair,
        dht_relay_url: Option<&Url>,
    ) -> Result<String, Error> {
        let challenge = self.get_challenge(&key_pair.public_key(), None);
        let signature = key_pair.sign(&challenge.unwrap().signable).to_string();
        let user_id = key_pair.to_z32();

        if self.homeserver_url.is_none() {
            self.homeserver_url = match self
                .resolver
                .resolve_homeserver(&key_pair.public_key(), dht_relay_url)
            {
                Ok(url) => Some(url),
                Err(e) => return Err(Error::FailedToResolveHomeserver(e)),
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
            Err(e) => return Err(Error::FailedToSendUserSignature(e)),
        }
    }

    /// Get challenge
    fn get_challenge(
        &mut self,
        public_key: &PublicKey,
        dht_relay_url: Option<&Url>,
    ) -> Result<Challenge, Error> {
        if self.homeserver_url.is_none() {
            self.homeserver_url = match self.resolver.resolve_homeserver(&public_key, dht_relay_url)
            {
                Ok(url) => Some(url),
                Err(e) => return Err(Error::FailedToResolveHomeserver(e)),
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
            Err(e) => return Err(Error::FailedToGetChallenge(e)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::*;
    use mainline::dht::Testnet;

    #[test]
    fn auth_walk_through() {
        let testnet = Testnet::new(10);

        let seed = b"it is a seed for key generation!";
        let key_pair: Keypair = DeterministicKeyGen::generate(Some(seed));
        let user_id = key_pair.to_z32();

        let server = create_homeserver_mock(
            user_id.to_string(),
            "repo_name".to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );

        let url = Url::parse(&server.url()).unwrap();
        let resolver = publish_url(&key_pair, &url, &testnet.bootstrap);

        let mut auth = Auth::new(resolver, None);

        // TEST SIGNUP
        let user_id = auth.signup(seed, None).unwrap();
        let session_id = "send_signature_signup".to_string();
        assert_eq!(user_id, user_id);
        assert_eq!(
            auth.homeserver_url,
            Some(Url::parse(&server.url()).unwrap())
        );
        assert_eq!(auth.session_id, Some(session_id));

        // TEST LOGOUT
        let res_session_id = auth.logout(&user_id).unwrap();
        assert_eq!(auth.session_id, None);
        assert_eq!(res_session_id, "send_signature_signup");

        // TEST SIGNUP AGAIN
        let resolver = Resolver::new(None, Some(&testnet.bootstrap));
        let mut auth = Auth::new(resolver, Some(Url::parse(&server.url()).unwrap()));

        let got_user_id = auth.signup(seed, None).unwrap();
        let session_id = "send_signature_signup".to_string();
        assert_eq!(got_user_id, user_id);
        assert_eq!(
            auth.homeserver_url,
            Some(Url::parse(&server.url()).unwrap())
        );
        assert_eq!(auth.session_id, Some(session_id));

        // TEST LOGOUT
        let res_session_id = auth.logout(&user_id).unwrap();
        assert_eq!(auth.session_id, None);
        assert_eq!(res_session_id, "send_signature_signup");

        // TEST LOGIN
        let res_user_id = auth.login(seed, None).unwrap();
        let session_id = "send_signature_login".to_string();
        assert_eq!(user_id, res_user_id);
        assert_eq!(
            auth.homeserver_url,
            Some(Url::parse(&server.url()).unwrap())
        );
        assert_eq!(auth.session_id, Some(session_id));

        // TEST SESSION
        let session = auth.session().unwrap();
        let session_id = "get_session".to_string();
        assert_eq!(session, "session".to_string());
        assert_eq!(auth.session_id, Some(session_id));
        assert_eq!(
            auth.homeserver_url,
            Some(Url::parse(&server.url()).unwrap())
        );
    }
}
