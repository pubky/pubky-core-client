use std::collections::HashMap;

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
/// It accepts optional homeserver URL and relay URL.
///
/// It has as a cache which matches {<userId>:(<homeserver_url, sesison_id>)}.
///
/// It has encapsulates an instance of a resolver to publish user's identity to the network, as
/// well as to lookup other user's homeservers
///
///
/// The CRUD operations for homeserver are performed using http requests.

pub struct Client<'a> {
    pub homeserver_url: Url, // own homeserver
    pub user_id: String,     // own user id
    seed: [u8; 32],
    homeservers_cache: HashMap<String, Auth<'a>>, // homervers of others
    dht_relay: Option<&'a Url>,
}

impl Client<'_> {
    pub fn new<'a>(
        seed: Option<[u8; 32]>,
        homeserver_url: Option<Url>,
        dht_relay: Option<&'a Url>,
        bootstrap: Option<&'a Vec<String>>,
    ) -> Client<'a> {
        let seed = seed.unwrap_or(crypto::random_bytes(32).try_into().unwrap());

        let resolver = Resolver::new(dht_relay, bootstrap);
        let mut auth = Auth::new(resolver, homeserver_url);

        let user_id = auth.signup(&seed, None).unwrap();
        let homeserver_url = auth.homeserver_url.clone().unwrap();

        let mut homeservers_cache = HashMap::new();
        homeservers_cache.insert(user_id.clone(), auth);

        Client {
            seed,
            homeservers_cache,
            homeserver_url,
            user_id,
            dht_relay,
        }
    }

    /* GENERAL LOGIC */

    /// Generate a new key pair
    pub fn generate_keypair(&self) -> crypto::Keypair {
        crypto::DeterministicKeyGen::generate(Some(&self.seed))
    }

    /// Get user id
    pub fn get_user_id(&self) -> String {
        let keypair = self.generate_keypair();
        keypair.to_z32()
    }

    /* "AUTH" RELATED LOGIC */
    /// login
    pub fn login(&mut self) -> Result<String, String> {
        self.homeservers_cache
            .get_mut(&self.user_id)
            .unwrap()
            .login(&self.seed, self.dht_relay)
    }

    /// logout
    pub fn logout(&mut self) -> Result<String, String> {
        self.homeservers_cache
            .get_mut(&self.user_id)
            .unwrap()
            .logout(&self.user_id)
    }

    /// session
    pub fn session(&mut self) -> Result<String, String> {
        self.homeservers_cache
            .get_mut(&self.user_id)
            .unwrap()
            .session()
    }

    /* "REPOS" RELATED LOGIC */

    /// Create repository for user
    pub fn create(&mut self, user_id: &str, repo_name: &str) -> Result<(), String> {
        let url = &self
            .homeservers_cache
            .get(user_id)
            .unwrap()
            .homeserver_url
            .clone()
            .unwrap()
            .join(&format!("/mvp/users/{}/repos/{}", user_id, repo_name))
            .unwrap();

        match request(
            Method::PUT,
            url.clone(),
            &mut self.homeservers_cache.get_mut(user_id).unwrap().session_id,
            None,
            None,
        ) {
            Ok(_) => Ok(()),
            Err(e) => return Err(e),
        }
    }

    /// Put data into user's repository and return URL to this repo
    pub fn put(
        &mut self,
        user_id: &str,
        repo_name: &str,
        path: &str,
        payload: &str,
    ) -> Result<Url, String> {
        let url = &self
            .homeservers_cache
            .get(user_id)
            .unwrap()
            .homeserver_url
            .clone()
            .unwrap()
            .join(&format!(
                "/mvp/users/{}/repos/{}/{}",
                user_id, repo_name, path
            ))
            .unwrap();

        println!("Calling: {}", url);

        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            "application/octet-stream".try_into().unwrap(),
        );
        headers.insert(
            "Content-Length",
            payload.len().to_string().try_into().unwrap(),
        );

        let response = request(
            Method::PUT,
            url.clone(),
            &mut self.homeservers_cache.get_mut(user_id).unwrap().session_id,
            Some(&headers),
            Some(payload.to_string()),
        );

        match response {
            Ok(_) => Ok(url.clone()),
            Err(e) => return Err(e),
        }
    }

    /// Get data from user's repository and return it as a JSON(?)
    pub fn get(&mut self, user_id: &str, repo_name: &str, path: &str) -> Result<String, String> {
        let url = &self
            .homeservers_cache
            .get(user_id)
            .unwrap()
            .homeserver_url
            .clone()
            .unwrap()
            .join(&format!(
                "/mvp/users/{}/repos/{}/{}",
                user_id, repo_name, path
            ))
            .unwrap();

        let response = request(
            Method::GET,
            url.clone(),
            &mut self.homeservers_cache.get_mut(user_id).unwrap().session_id,
            None,
            None,
        );

        match response {
            Ok(body) => Ok(body),
            Err(e) => return Err(e),
        }
    }

    /// Delete data from user's repository
    pub fn delete(&mut self, user_id: &str, repo_name: &str, path: &str) -> Result<(), String> {
        let url = &self
            .homeservers_cache
            .get(user_id)
            .unwrap()
            .homeserver_url
            .clone()
            .unwrap()
            .join(&format!(
                "/mvp/users/{}/repos/{}/{}",
                user_id, repo_name, path
            ))
            .unwrap();

        let response = request(
            Method::DELETE,
            url.clone(),
            &mut self.homeservers_cache.get_mut(user_id).unwrap().session_id,
            None,
            None,
        );

        match response {
            Ok(_) => Ok(()),
            Err(e) => return Err(e),
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
        let seed = b"it is a seed for key generation!";

        let key_pair: Keypair = DeterministicKeyGen::generate(Some(seed));
        let user_id = key_pair.to_z32();
        let server = create_homeserver_mock(
            user_id.to_string(),
            "repo_name".to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );
        let _ = publish_url(
            &key_pair,
            &Url::parse(&server.url()).unwrap(),
            &testnet.bootstrap,
        );

        let client = Client::new(Some(seed.clone()), None, None, Some(&testnet.bootstrap));

        assert_eq!(client.homeservers_cache.len(), 1);
        assert_eq!(
            client
                .homeservers_cache
                .get(&user_id)
                .unwrap()
                .homeserver_url,
            Some(Url::parse(&server.url()).unwrap())
        );
        assert_eq!(
            client.homeservers_cache.get(&user_id).unwrap().session_id,
            Some("send_signature_signup".to_string())
        );
    }

    #[test]
    fn test_client_create() {
        let seed = b"it is a seed for key generation!";
        let testnet = Testnet::new(10);

        let key_pair: Keypair = DeterministicKeyGen::generate(Some(seed));
        let user_id = key_pair.to_z32();
        let repo_name = "test_repo";

        let server = create_homeserver_mock(
            user_id.to_string(),
            repo_name.to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );
        let _ = publish_url(
            &key_pair,
            &Url::parse(&server.url()).unwrap(),
            &testnet.bootstrap,
        );
        let mut client = Client::new(Some(seed.clone()), None, None, Some(&testnet.bootstrap));

        let result = client.create(&user_id, repo_name);

        assert_eq!(result, Ok(()));
        assert_eq!(client.homeservers_cache.len(), 1);
        assert_eq!(
            client
                .homeservers_cache
                .get(&user_id)
                .unwrap()
                .homeserver_url,
            Some(Url::parse(&server.url()).unwrap())
        );
        assert_eq!(
            client.homeservers_cache.get(&user_id).unwrap().session_id,
            Some("create_repo".to_string())
        );
    }

    #[test]
    fn test_client_put() {
        let testnet = Testnet::new(10);
        let seed = b"it is a seed for key generation!";

        let key_pair: Keypair = DeterministicKeyGen::generate(Some(seed));
        let user_id = key_pair.to_z32();

        let repo_name = "test_repo";
        let folder_path = "test_path";
        let server = create_homeserver_mock(
            user_id.to_string(),
            repo_name.to_string(),
            folder_path.to_string(),
            "test_payload".to_string(),
        );

        let _ = publish_url(
            &key_pair,
            &Url::parse(&server.url()).unwrap(),
            &testnet.bootstrap,
        );

        let mut client = Client::new(Some(seed.clone()), None, None, Some(&testnet.bootstrap));

        let result = client.put(&user_id, repo_name, &folder_path, "test_payload");

        assert_eq!(
            result.unwrap(),
            Url::parse(&server.url())
                .unwrap()
                .join(
                    format!("/mvp/users/{}/repos/{}/{}", user_id, repo_name, folder_path).as_str()
                )
                .unwrap()
        );
        assert_eq!(client.homeservers_cache.len(), 1);
        assert_eq!(
            client
                .homeservers_cache
                .get(&user_id)
                .unwrap()
                .homeserver_url,
            Some(Url::parse(&server.url()).unwrap())
        );
        assert_eq!(
            client.homeservers_cache.get(&user_id).unwrap().session_id,
            Some("create_folder".to_string())
        );
    }

    #[test]
    fn test_client_get() {
        let testnet = Testnet::new(10);

        let seed = b"it is a seed for key generation!";
        let key_pair: Keypair = DeterministicKeyGen::generate(Some(seed));
        let user_id = key_pair.to_z32();

        let repo_name = "test_repo";
        let folder_path = "test_path";
        let data = "test_payload";

        let server = create_homeserver_mock(
            user_id.to_string(),
            repo_name.to_string(),
            folder_path.to_string(),
            data.to_string(),
        );

        let _ = publish_url(
            &key_pair,
            &Url::parse(&server.url()).unwrap(),
            &testnet.bootstrap,
        );

        let mut client = Client::new(Some(seed.clone()), None, None, Some(&testnet.bootstrap));

        let result = client.get(&user_id, repo_name, &folder_path);

        assert_eq!(result.unwrap(), data.to_string());
        assert_eq!(client.homeservers_cache.len(), 1);
        assert_eq!(
            client
                .homeservers_cache
                .get(&user_id)
                .unwrap()
                .homeserver_url,
            Some(Url::parse(&server.url()).unwrap())
        );
        assert_eq!(
            client.homeservers_cache.get(&user_id).unwrap().session_id,
            Some("get_data".to_string())
        );
    }

    #[test]
    fn test_client_delete() {
        let testnet = Testnet::new(10);

        let seed = b"it is a seed for key generation!";
        let key_pair: Keypair = DeterministicKeyGen::generate(Some(seed));
        let user_id = key_pair.to_z32();
        let repo_name = "test_repo";
        let folder_path = "test_path";

        let server = create_homeserver_mock(
            user_id.to_string(),
            repo_name.to_string(),
            folder_path.to_string(),
            "data".to_string(),
        );

        let _ = publish_url(
            &key_pair,
            &Url::parse(&server.url()).unwrap(),
            &testnet.bootstrap,
        );

        let mut client = Client::new(Some(seed.clone()), None, None, Some(&testnet.bootstrap));

        let result = client.delete(&user_id, repo_name, &folder_path);

        assert!(result.is_ok());
        assert_eq!(client.homeservers_cache.len(), 1);
        assert_eq!(
            client
                .homeservers_cache
                .get(&user_id)
                .unwrap()
                .homeserver_url,
            Some(Url::parse(&server.url()).unwrap())
        );
        assert_eq!(
            client.homeservers_cache.get(&user_id).unwrap().session_id,
            Some("delete_data".to_string())
        );
    }
}
