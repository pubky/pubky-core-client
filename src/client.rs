use crate::error::ClientError as Error;

use std::collections::HashMap;

use crate::helpers::Path;
use crate::transport::{
    auth::Auth,
    crypto,
    http::{request, HeaderMap, Method, Url},
    resolver::Resolver,
};

/// This is the pubky client class. It is used for accessing pubky infrastructure for CRUD options
/// over user's data in pubky network.
///
/// Client accepts optional seed for pubky key generation.
/// It accepts optional homeserver URL.
///
/// It has as a cache which matches {userId:(homeserver_url, sesison_id)}.
///
/// It has encapsulates an instance of a resolver to publish user's identity to the network, as
/// well as to lookup other user's homeservers
///
///
/// The CRUD operations for homeserver are performed using http requests.

pub struct Client<'a> {
    homeservers_cache: HashMap<String, Auth<'a>>,
    bootstrap: Option<&'a Vec<String>>,
}

impl Client<'_> {
    pub fn new<'a>(bootstrap: Option<&'a Vec<String>>) -> Client<'a> {
        Client {
            homeservers_cache: HashMap::new(),
            bootstrap,
        }
    }

    pub fn generate_seed() -> [u8; 32] {
        crypto::random_bytes(32).try_into().unwrap()
    }

    /* "AUTH" RELATED LOGIC */
    /// signup
    pub fn signup(
        &mut self,
        seed: [u8; 32],
        homeserver_url: Option<Url>,
    ) -> Result<String, Error> {
        let resolver = Resolver::new(self.bootstrap);
        let mut auth = Auth::new(resolver, homeserver_url);

        match auth.signup(&seed) {
            Ok(user_id) => {
                let _ = &self.homeservers_cache.insert(user_id.clone(), auth);
                Ok(user_id)
            }
            Err(e) => return Err(Error::FailedToSignup(e)),
        }
    }

    /// login
    pub fn login(
        &mut self,
        seed: [u8; 32],
        homeserver_url: Option<Url>,
    ) -> Result<String, Error> {
        let resolver = Resolver::new(self.bootstrap);
        let mut auth = Auth::new(resolver, homeserver_url);

        match auth.login(&seed) {
            Ok(user_id) => {
                let _ = &self.homeservers_cache.insert(user_id.clone(), auth);
                Ok(user_id)
            }
            Err(e) => return Err(Error::FailedToLogin(e)),
        }
    }

    /// logout
    pub fn logout(&mut self, user_id: String) -> Result<String, Error> {
        match self
            .homeservers_cache
            .get_mut(&user_id)
            .ok_or(Error::UserNotSignedUp)?
            .logout(&user_id)
        {
            Ok(session_id) => {
                let _ = self.homeservers_cache.remove(&user_id);
                Ok(session_id)
            }
            Err(e) => Err(Error::FailedToLogout(e)),
        }
    }

    /// session
    pub fn session(&mut self, user_id: String) -> Result<String, Error> {
        match self
            .homeservers_cache
            .get_mut(&user_id)
            .ok_or(Error::UserNotSignedUp)?
            .session()
        {
            Ok(session) => Ok(session),
            Err(e) => Err(Error::FailedToRetrieveSession(e)),
        }
    }

    /// gets homeserver url
    pub fn get_home_server_url(&self, user_id: &str) -> Result<Url, Error> {
        Ok(self
            .homeservers_cache
            .get(user_id)
            .ok_or(Error::UserNotSignedUp)?
            .homeserver_url
            .clone()
            .unwrap())
    }

    /// gets current session
    pub fn get_current_session(&self, user_id: &str) -> Result<String, Error> {
        Ok(self
            .homeservers_cache
            .get(user_id)
            .ok_or(Error::UserNotSignedUp)?
            .session_id
            .clone()
            .unwrap())
    }

    fn get_mut_session(&mut self, user_id: &str) -> Result<&mut Option<String>, Error> {
        Ok(&mut self
            .homeservers_cache
            .get_mut(user_id)
            .ok_or(Error::UserNotSignedUp)?
            .session_id)
    }

    /* "REPOS" RELATED LOGIC */

    /// Create repository for user
    pub fn create(&mut self, user_id: &str, repo_name: &str) -> Result<(), Error> {
        let url = &self
            .get_home_server_url(user_id)?
            .join(&Path::get_repo_string(user_id, repo_name, None))
            .unwrap();

        match request(
            Method::PUT,
            url.clone(),
            self.get_mut_session(user_id)?,
            None,
            None,
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::FailedToCreateRepository(e)),
        }
    }

    /// Put data into user's repository and return URL to this repo
    pub fn put(
        &mut self,
        user_id: &str,
        repo_name: &str,
        path: &str,
        payload: &str,
    ) -> Result<Url, Error> {
        let url = &self
            .get_home_server_url(user_id)?
            .join(&Path::get_repo_string(user_id, repo_name, Some(path)))
            .unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            "application/octet-stream".try_into().unwrap(),
        );
        headers.insert(
            "Content-Length",
            payload.len().to_string().try_into().unwrap(),
        );

        match request(
            Method::PUT,
            url.clone(),
            self.get_mut_session(user_id)?,
            Some(&headers),
            Some(payload.to_string()),
        ) {
            Ok(_) => Ok(url.clone()),
            Err(e) => Err(Error::FailedToStoreData(e)),
        }
    }

    /// Get data from user's repository and return it as a JSON(?)
    pub fn get(&mut self, user_id: &str, repo_name: &str, path: &str) -> Result<String, Error> {
        let url = &self
            .get_home_server_url(user_id)?
            .join(&Path::get_repo_string(user_id, repo_name, Some(path)))
            .unwrap();

        match request(
            Method::GET,
            url.clone(),
            self.get_mut_session(user_id)?,
            None,
            None,
        ) {
            Ok(body) => Ok(body),
            Err(e) => Err(Error::FailedToRetrieveData(e)),
        }
    }

    /// Delete data from user's repository
    pub fn delete(&mut self, user_id: &str, repo_name: &str, path: &str) -> Result<(), Error> {
        let url = &self
            .get_home_server_url(user_id)?
            .join(&Path::get_repo_string(user_id, repo_name, Some(path)))
            .unwrap();

        match request(
            Method::DELETE,
            url.clone(),
            self.get_mut_session(user_id)?,
            None,
            None,
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::FailedToDeleteData(e)),
        }
    }

    //     /// List data in user's repository
    //     /*
    //     ListOption {
    //          reverse: bool,
    //          offset: usize,
    //          limit: usize,
    //     }
    //     */
    //     pub fn list (&mut self, user_id: &str, repo_name: &str, path: &str, opts: Option<ListOption>) -> Result<Vec<String>, String> { }
    //
    //
    //     /// Query data in user's repository
    //     /*
    //     // Maybe can repurpose ListOption
    //     QueryOptions {
    //          reverse: bool,
    //          start: usize,
    //          end: usize,
    //          limit: usize,
    //          reverse: bool,
    //     }
    //     */
    //     pub fn query (&mut self, user_id: &str, repo_name: &str, query: Option<QueryOptions>) -> Result<Vec<String>, String> { }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use crate::transport::crypto::{DeterministicKeyGen, Keypair};
    use mainline::dht::Testnet;

    #[test]
    fn test_client_new() {
        let testnet = Testnet::new(10);
        let seed = Client::generate_seed();

        let key_pair: Keypair = DeterministicKeyGen::generate(Some(&seed));
        let user_id = key_pair.to_z32();
        let (mut server, _homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            "repo_name".to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );

        let client = Client::new(Some(&testnet.bootstrap));

        assert_eq!(client.homeservers_cache.len(), 0);

        server.reset();
    }

    #[test]
    fn test_client_signup_with_seed() {
        let testnet = Testnet::new(10);
        let seed = Client::generate_seed();

        let key_pair: Keypair = DeterministicKeyGen::generate(Some(&seed));
        let user_id = key_pair.to_z32();
        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            "repo_name".to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );
        // since homeserver_url is not to be provided it will be resolved from the the seed
        // thus needs to be published beforehand
        let _ = publish_url(&key_pair, &homeserver_url, &testnet.bootstrap);

        let mut client = Client::new(Some(&testnet.bootstrap));

        assert_eq!(client.homeservers_cache.len(), 0);

        client.signup(seed, None).unwrap();

        assert_eq!(client.homeservers_cache.len(), 1);
        assert_eq!(
            client.get_home_server_url(&user_id).unwrap(),
            homeserver_url
        );
        assert_eq!(
            client.get_current_session(&user_id).unwrap(),
            "send_signature_signup".to_string()
        );

        server.reset();
    }

    #[test]
    fn test_client_signup_with_seed_url() {
        let testnet = Testnet::new(10);
        let seed = Client::generate_seed();

        let key_pair: Keypair = DeterministicKeyGen::generate(Some(&seed));
        let user_id = key_pair.to_z32();
        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            "repo_name".to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );

        let mut client = Client::new(Some(&testnet.bootstrap));

        assert_eq!(client.homeservers_cache.len(), 0);

        client
            .signup(seed, Some(homeserver_url.clone()))
            .unwrap();

        assert_eq!(client.homeservers_cache.len(), 1);
        assert_eq!(
            client.get_home_server_url(&user_id).unwrap(),
            homeserver_url
        );
        assert_eq!(
            client.get_current_session(&user_id).unwrap(),
            "send_signature_signup".to_string()
        );

        server.reset();
    }

    #[test]
    fn test_client_login_with_seed() {
        let testnet = Testnet::new(10);
        let seed = Client::generate_seed();

        let key_pair: Keypair = DeterministicKeyGen::generate(Some(&seed));
        let user_id = key_pair.to_z32();
        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            "repo_name".to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );
        // since homeserver_url is not to be provided it will be resolved from the the seed
        // thus needs to be published beforehand
        let _ = publish_url(&key_pair, &homeserver_url, &testnet.bootstrap);

        let mut client = Client::new(Some(&testnet.bootstrap));

        assert_eq!(client.homeservers_cache.len(), 0);

        client.login(seed, None).unwrap();

        assert_eq!(
            client.get_home_server_url(&user_id).unwrap(),
            homeserver_url
        );
        assert_eq!(
            client.get_current_session(&user_id).unwrap(),
            "send_signature_login".to_string()
        );

        server.reset();
    }

    #[test]
    fn test_client_login_with_seed_url() {
        let testnet = Testnet::new(10);
        let seed = Client::generate_seed();

        let key_pair: Keypair = DeterministicKeyGen::generate(Some(&seed));
        let user_id = key_pair.to_z32();
        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            "repo_name".to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );
        let mut client = Client::new(Some(&testnet.bootstrap));

        assert_eq!(client.homeservers_cache.len(), 0);

        client
            .login(seed, Some(homeserver_url.clone()))
            .unwrap();

        assert_eq!(client.homeservers_cache.len(), 1);
        assert_eq!(
            client.get_home_server_url(&user_id).unwrap(),
            homeserver_url
        );
        assert_eq!(
            client.get_current_session(&user_id).unwrap(),
            "send_signature_login".to_string()
        );

        server.reset();
    }

    #[test]
    fn test_client_create() {
        let seed = Client::generate_seed();
        let testnet = Testnet::new(10);

        let key_pair: Keypair = DeterministicKeyGen::generate(Some(&seed));
        let user_id = key_pair.to_z32();
        let repo_name = "test_repo";

        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            repo_name.to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );
        let mut client = Client::new(Some(&testnet.bootstrap));
        let user_id = client
            .login(seed, Some(homeserver_url.clone()))
            .unwrap();

        let result = client.create(&user_id, repo_name);

        assert!(result.is_ok());
        assert_eq!(client.homeservers_cache.len(), 1);
        assert_eq!(
            client.get_home_server_url(&user_id).unwrap(),
            homeserver_url
        );
        assert_eq!(
            client.get_current_session(&user_id).unwrap(),
            "create_repo".to_string()
        );

        server.reset();
    }

    #[test]
    fn test_client_put() {
        let testnet = Testnet::new(10);
        let seed = Client::generate_seed();

        let key_pair: Keypair = DeterministicKeyGen::generate(Some(&seed));
        let user_id = key_pair.to_z32();

        let repo_name = "test_repo";
        let folder_path = "test_path";
        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            repo_name.to_string(),
            folder_path.to_string(),
            "test_payload".to_string(),
        );
        let mut client = Client::new(Some(&testnet.bootstrap));
        let user_id = client
            .login(seed, Some(homeserver_url.clone()))
            .unwrap();

        let result = client.put(&user_id, repo_name, folder_path, "test_payload");

        assert_eq!(
            result.unwrap(),
            homeserver_url
                .join(&Path::get_repo_string(
                    &user_id,
                    repo_name,
                    Some(folder_path)
                ))
                .unwrap()
        );
        assert_eq!(client.homeservers_cache.len(), 1);
        assert_eq!(
            client.get_home_server_url(&user_id).unwrap(),
            homeserver_url
        );
        assert_eq!(
            client.get_current_session(&user_id).unwrap(),
            "create_folder".to_string()
        );

        server.reset();
    }

    #[test]
    fn test_client_get() {
        let testnet = Testnet::new(10);

        let seed = Client::generate_seed();
        let key_pair: Keypair = DeterministicKeyGen::generate(Some(&seed));
        let user_id = key_pair.to_z32();

        let repo_name = "test_repo";
        let folder_path = "test_path";
        let data = "test_payload";

        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            repo_name.to_string(),
            folder_path.to_string(),
            data.to_string(),
        );
        let mut client = Client::new(Some(&testnet.bootstrap));
        let user_id = client
            .login(seed, Some(homeserver_url.clone()))
            .unwrap();

        let result = client.get(&user_id, repo_name, folder_path);

        assert_eq!(result.unwrap(), data.to_string());
        assert_eq!(client.homeservers_cache.len(), 1);
        assert_eq!(
            client.get_home_server_url(&user_id).unwrap(),
            homeserver_url
        );
        assert_eq!(
            client.get_current_session(&user_id).unwrap(),
            "get_data".to_string()
        );

        server.reset();
    }

    #[test]
    fn test_client_delete() {
        let testnet = Testnet::new(10);

        let seed = Client::generate_seed();
        let key_pair: Keypair = DeterministicKeyGen::generate(Some(&seed));
        let user_id = key_pair.to_z32();
        let repo_name = "test_repo";
        let folder_path = "test_path";

        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            repo_name.to_string(),
            folder_path.to_string(),
            "data".to_string(),
        );
        let mut client = Client::new(Some(&testnet.bootstrap));
        let user_id = client
            .login(seed, Some(homeserver_url.clone()))
            .unwrap();

        let result = client.delete(&user_id, repo_name, folder_path);

        assert!(result.is_ok());
        assert_eq!(client.homeservers_cache.len(), 1);
        assert_eq!(
            client.get_home_server_url(&user_id).unwrap(),
            homeserver_url
        );
        assert_eq!(
            client.get_current_session(&user_id).unwrap(),
            "delete_data".to_string()
        );

        server.reset();
    }
}
