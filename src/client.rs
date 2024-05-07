use crate::error::ClientError as Error;

use std::collections::HashMap;

use crate::helpers::Path;
use crate::transport::{
    auth::Auth,
    http::{request, HeaderMap, Method, Url},
    resolver::Resolver,
};

/// This is the pubky client class. It is used for accessing pubky infrastructure for CRUD options over user's data in pubky network.
///
/// Client accepts optional list of bootstraping DHT nodes to resolve user's `homeserver_url`.
///
/// It encapsulates an instance of `Auth` object to publish user's identity to the network, to lookup other user's homeservers which acts as a cache that matches `userId` to `homeserver_url` and corresponding `sesison_id`.
///
/// The CRUD operations for homeserver are performed using http requests.
pub struct Client {
    homeservers_cache: HashMap<String, Auth>,
    bootstrap: Option<Vec<String>>,
}

impl Client {
    /// Create a new instance of Client
    ///
    /// # Parameters
    /// * `bootstrap` - Optional pointer to the list of bootstraping DHT nodes to resolve user's homeserver url.
    ///
    /// # Returns
    /// * `Client` - New instance of `Client`
    ///
    /// # Example
    /// ```
    /// use pubky_core_client::client::Client;
    /// # use mainline::dht::Testnet;
    ///
    /// let bootstrap: Option<Vec<String>> = None;
    /// # let testnet = Testnet::new(10);
    /// # let bootstrap = Some(testnet.bootstrap);
    ///
    /// let client = Client::new(bootstrap);
    /// ```
    pub fn new(bootstrap: Option<Vec<String>>) -> Client {
        Client {
            homeservers_cache: HashMap::new(),
            bootstrap,
        }
    }

    /* "AUTH" RELATED LOGIC */

    /// Signup to the homeserver using seed either with or without homeserver url. In case if `homeserver_url` is not provided it will be resolved from the `seed`'s public key. URL will be republished to DHT using [Pkarr](https://github.com/Nuhvi/pkarr/)
    ///
    /// # Parameters
    /// * `seed` - 32 bytes seed to generate user's identity
    /// * `homeserver_url` - Optional URL of the homeserver_url
    ///
    /// # Returns
    /// * `Result<String, Error>` - User's identity
    ///
    /// # Errors
    /// * `Error::FailedToSignup` - If signup fails
    ///
    /// # Example
    /// ```
    /// use pubky_core_client::client::Client;
    /// use pubky_core_client::utils::generate_seed;
    /// use url::Url;
    /// # use mainline::dht::Testnet;
    ///
    /// let bootstrap: Option<Vec<String>> = None;
    /// let homeserver_url: Option<Url> = None;
    /// # let testnet = Testnet::new(10);
    /// # let bootstrap = Some(testnet.bootstrap);
    ///
    /// // Client needs to be mutable to perform signup as it will update the cache with user's identity
    /// let mut client = Client::new(None);
    /// let seed = generate_seed();
    ///
    /// match client.signup(seed, homeserver_url) {
    ///      Ok(user_id) => println!("{user_id}"),
    ///      Err(e) => println!("{e:?}")
    /// }
    /// ```
    pub fn signup(&mut self, seed: [u8; 32], homeserver_url: Option<Url>) -> Result<String, Error> {
        let resolver = Resolver::new(self.bootstrap.clone());
        let mut auth = Auth::new(resolver, homeserver_url);

        match auth.signup(&seed) {
            Ok(user_id) => {
                let _ = &self.homeservers_cache.insert(user_id.clone(), auth);
                Ok(user_id)
            }
            Err(e) => return Err(Error::FailedToSignup(e)),
        }
    }

    /// Login to the homeserver using `seed` either with or without `homeserver_url`. In case if homeserver url is not provided it will be resolved from the seed's public key. URL will be republished to DHT using [Pkarr](http://github.com/Nhubei/pkarr/)
    ///
    /// # Parameters
    /// * `seed` - 32 bytes seed to generate user's identity
    /// * `homeserver_url` - Optional URL of the homeserver_url
    ///
    /// # Returns
    /// * `Result<String, Error>` - User's identity
    ///
    /// # Errors
    /// * `Error::FailedToLogin` - If login fails
    ///
    /// # Example
    /// ```
    /// use pubky_core_client::client::Client;
    /// use pubky_core_client::utils::generate_seed;
    /// use url::Url;
    /// # use mainline::dht::Testnet;
    ///
    /// let bootstrap: Option<Vec<String>> = None;
    /// let homeserver_url: Option<Url> = None;
    /// # let testnet = Testnet::new(10);
    /// # let bootstrap = Some(testnet.bootstrap);
    ///
    /// // Client needs to be mutable to perform signup as it will update the cache with user's identity
    /// let mut client = Client::new(None);
    /// let seed = generate_seed();
    ///
    /// match client.login(seed, homeserver_url) {
    ///      Ok(user_id) => println!("{user_id}"),
    ///      Err(e) => println!("{e:?}")
    /// }
    /// ```
    pub fn login(&mut self, seed: [u8; 32], homeserver_url: Option<Url>) -> Result<String, Error> {
        let resolver = Resolver::new(self.bootstrap.clone());
        let mut auth = Auth::new(resolver, homeserver_url);

        match auth.login(&seed) {
            Ok(user_id) => {
                let _ = &self.homeservers_cache.insert(user_id.clone(), auth);
                Ok(user_id)
            }
            Err(e) => return Err(Error::FailedToLogin(e)),
        }
    }

    /// Logout from the homeserver associated with the `user_id`. It will remove the `user_id` from the internal cache. If the `user_id` is not found in the cache it will return an error. If the logout fails it will return an error.
    ///
    /// # Parameters
    /// * `user_id` - User's identity
    ///
    /// # Returns
    /// * `Result<String, Error>` - Session ID which was associated with used
    ///
    /// # Errors
    /// * `Error::UserNotSignedUp` - If user is not signed up
    /// * `Error::FailedToLogout` - If logout fails
    ///
    /// # Example
    /// ```
    /// use pubky_core_client::client::Client;
    /// use pubky_core_client::utils::generate_seed;
    /// use url::Url;
    /// # use mainline::dht::Testnet;
    ///
    /// let bootstrap: Option<Vec<String>> = None;
    /// let homeserver_url: Option<Url> = None;
    /// # let testnet = Testnet::new(10);
    /// # let bootstrap = Some(testnet.bootstrap);
    ///
    /// // Client needs to be mutable to perform signup as it will update the cache with user's identity
    /// let mut client = Client::new(None);
    /// let seed = generate_seed();
    ///
    /// match client.login(seed, homeserver_url) {
    ///      Ok(user_id) => {
    ///           client.logout(&user_id);
    ///      },
    ///      Err(e) => println!("{e:?}")
    /// };
    /// ```
    pub fn logout(&mut self, user_id: &str) -> Result<String, Error> {
        match self
            .homeservers_cache
            .get_mut(user_id)
            .ok_or(Error::UserNotSignedUp)?
            .logout(user_id)
        {
            Ok(session_id) => {
                let _ = self.homeservers_cache.remove(user_id);
                Ok(session_id)
            }
            Err(e) => Err(Error::FailedToLogout(e)),
        }
    }

    /// Requst session from the homeserver associated with the `user_id`. If the `user_id` is not found in the cache it will return an error. If the session retrieval fails it will return an error.
    ///
    /// # Parameters
    /// * `user_id` - User's identity
    ///
    /// # Returns
    /// // FIXME: it should be a session object
    /// * `Result<String, Error>` - Session string which was associated with user
    ///
    /// # Errors
    /// * `Error::UserNotSignedUp` - If user is not signed up
    /// * `Error::FailedToRetrieveSession` - If session retrieval fails
    ///
    /// # Example
    /// ```
    /// use pubky_core_client::client::Client;
    /// use pubky_core_client::utils::generate_seed;
    /// use url::Url;
    /// # use mainline::dht::Testnet;
    ///
    /// let bootstrap: Option<Vec<String>> = None;
    /// let homeserver_url: Option<Url> = None;
    /// # let testnet = Testnet::new(10);
    /// # let bootstrap = Some(testnet.bootstrap);
    ///
    /// // Client needs to be mutable to perform signup as it will update the cache with user's identity
    /// let mut client = Client::new(None);
    /// let seed = generate_seed();
    ///
    /// match client.login(seed, homeserver_url) {
    ///      Ok(user_id) => {
    ///           client.session(&user_id);
    ///      },
    ///      Err(e) => println!("{e:?}")
    /// };
    /// ```
    pub fn session(&mut self, user_id: &str) -> Result<String, Error> {
        match self
            .homeservers_cache
            .get_mut(user_id)
            .ok_or(Error::UserNotSignedUp)?
            .session()
        {
            Ok(session) => Ok(session),
            Err(e) => Err(Error::FailedToRetrieveSession(e)),
        }
    }

    /// Geet `homeserver_url` currently associated with `user_id`
    ///
    /// # Parameters
    /// * `user_id` - User's identity
    ///
    /// # Returns
    /// * `Result<Url, Error>` - URL of the `homeserver_url`
    ///
    /// # Errors
    /// * `Error::UserNotSignedUp` - If user is not signed up
    ///
    /// # Example
    /// ```
    /// use pubky_core_client::client::Client;
    /// use pubky_core_client::utils::generate_seed;
    /// use url::Url;
    /// # use mainline::dht::Testnet;
    ///
    /// let bootstrap: Option<Vec<String>> = None;
    /// let homeserver_url: Option<Url> = None;
    /// # let testnet = Testnet::new(10);
    /// # let bootstrap = Some(testnet.bootstrap);
    ///
    /// // Client needs to be mutable to perform signup as it will update the cache with user's identity
    /// let mut client = Client::new(None);
    /// let seed = generate_seed();
    ///
    /// match client.login(seed, homeserver_url) {
    ///      Ok(user_id) => {
    ///           client.get_home_server_url(&user_id);
    ///      },
    ///      Err(e) => println!("{e:?}")
    /// };
    /// ```
    pub fn get_home_server_url(&self, user_id: &str) -> Result<Url, Error> {
        Ok(self
            .homeservers_cache
            .get(user_id)
            .ok_or(Error::UserNotSignedUp)?
            .homeserver_url
            .clone()
            .unwrap())
    }

    /// Gets `session` string currently associated with `user_id`
    ///
    /// # Parameters
    /// * `user_id` - User's identity
    ///
    /// # Returns
    /// * `Result<String, Error>` - Session string
    ///
    /// # Errors
    /// * `Error::UserNotSignedUp` - If user is not signed up
    ///
    /// # Example
    /// ```
    /// use pubky_core_client::client::Client;
    /// use pubky_core_client::utils::generate_seed;
    /// use url::Url;
    /// # use mainline::dht::Testnet;
    ///
    /// let bootstrap: Option<Vec<String>> = None;
    /// let homeserver_url: Option<Url> = None;
    /// # let testnet = Testnet::new(10);
    /// # let bootstrap = Some(testnet.bootstrap);
    ///
    /// // Client needs to be mutable to perform signup as it will update the cache with user's identity
    /// let mut client = Client::new(None);
    /// let seed = generate_seed();
    ///
    /// match client.login(seed, homeserver_url) {
    ///      Ok(user_id) => {
    ///           client.get_current_session(&user_id);
    ///      },
    ///      Err(e) => println!("{e:?}")
    /// };
    /// ```
    pub fn get_current_session(&self, user_id: &str) -> Result<String, Error> {
        Ok(self
            .homeservers_cache
            .get(user_id)
            .ok_or(Error::UserNotSignedUp)?
            .session_id
            .clone()
            .unwrap())
    }

    /// Helper method to get mutable reference to the `session` string currently associated with `user_id`
    fn get_mut_session(&mut self, user_id: &str) -> Result<&mut Option<String>, Error> {
        Ok(&mut self
            .homeservers_cache
            .get_mut(user_id)
            .ok_or(Error::UserNotSignedUp)?
            .session_id)
    }

    /// Helper method to get URL path for the given `user_id`, `repo_name` and `path`
    fn get_url_path(
        &self,
        user_id: &str,
        repo_name: &str,
        path: Option<&str>,
    ) -> Result<Url, Error> {
        let url = &self.get_home_server_url(user_id)?;
        let path = Path::get_repo_string(user_id, repo_name, path);
        let url = url.join(&path);
        match url {
            Ok(url) => Ok(url),
            Err(_) => Err(Error::InvalidInputForUrl),
        }
    }

    /* "REPOS" RELATED LOGIC */

    /// Create repository as a user on the homeserver. It will return an error if the repository creation fails.
    ///
    /// # Parameters
    /// * `user_id` - User's identity
    /// * `repo_name` - Name of the repository
    ///
    /// # Returns
    /// * `Result<(), Error>` - Empty Result
    ///
    /// # Errors
    /// * `Error::FailedToCreateRepository` - If repository creation fails
    /// * `Error::UserNotSignedUp` - If user is not signed up
    /// * `Error::InvalidInputForUrl` - If input parameters can not be converted to a valid URL
    ///
    /// # Example
    /// ```
    /// use pubky_core_client::client::Client;
    /// use pubky_core_client::utils::generate_seed;
    /// use url::Url;
    /// # use mainline::dht::Testnet;
    ///
    /// let bootstrap: Option<Vec<String>> = None;
    /// let homeserver_url: Option<Url> = None;
    /// # let testnet = Testnet::new(10);
    /// # let bootstrap = Some(testnet.bootstrap);
    ///
    /// // Client needs to be mutable to perform signup as it will update the cache with user's identity
    /// let mut client = Client::new(None);
    /// let seed = generate_seed();
    ///
    /// let repo_name = "test_repo";
    /// match client.login(seed, homeserver_url) {
    ///      Ok(user_id) => {
    ///           client.create(&user_id, repo_name);
    ///      },
    ///      Err(e) => println!("{e:?}")
    /// };
    /// ```
    pub fn create(&mut self, user_id: &str, repo_name: &str) -> Result<(), Error> {
        let url = &self.get_url_path(user_id, repo_name, None)?;

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

    /// Put data under given path into user's repository and return URL to this data.
    ///
    /// # Parameters
    /// * `user_id` - User's identity
    /// * `repo_name` - Name of the repository
    /// * `path` - Path to the data
    /// * `payload` - Data to be stored
    ///
    /// # Returns
    /// * `Result<Url, Error>` - URL of the data
    ///
    /// # Errors
    /// * `Error::FailedToStoreData` - If storing data fails
    /// * `Error::UserNotSignedUp` - If user is not signed up
    /// * `Error::InvalidInputForUrl` - If input parameters can not be converted to a valid URL
    ///
    /// # Example
    /// ```
    /// use pubky_core_client::client::Client;
    /// use pubky_core_client::utils::generate_seed;
    /// use url::Url;
    /// # use mainline::dht::Testnet;
    ///
    /// let bootstrap: Option<Vec<String>> = None;
    /// let homeserver_url: Option<Url> = None;
    /// # let testnet = Testnet::new(10);
    /// # let bootstrap = Some(testnet.bootstrap);
    ///
    /// // Client needs to be mutable to perform signup as it will update the cache with user's identity
    /// let mut client = Client::new(None);
    /// let seed = generate_seed();
    ///
    /// let repo_name = "test_repo";
    /// let path = "test_path";
    /// let payload = "{ \"data\": \"test_data\" }}";
    /// match client.login(seed, homeserver_url) {
    ///      Ok(user_id) => {
    ///           client.put(&user_id, repo_name, path, payload);
    ///      },
    ///      Err(e) => println!("{e:?}")
    /// };
    /// ```
    pub fn put(
        &mut self,
        user_id: &str,
        repo_name: &str,
        path: &str,
        payload: &str,
    ) -> Result<Url, Error> {
        let url = &self.get_url_path(user_id, repo_name, Some(path))?;

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
    use crate::utils::{generate_keypair, generate_seed, get_user_id};
    use mainline::dht::Testnet;

    #[test]
    fn test_client_new() {
        let testnet = Testnet::new(10);
        let seed = generate_seed();

        let user_id = get_user_id(Some(&seed));
        let (mut server, _homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            "repo_name".to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );

        let client = Client::new(Some(testnet.bootstrap));

        assert_eq!(client.homeservers_cache.len(), 0);

        server.reset();
    }

    #[test]
    fn test_client_signup_with_seed() {
        let testnet = Testnet::new(10);
        let seed = generate_seed();

        let key_pair = generate_keypair(Some(&seed));
        let user_id = get_user_id(Some(&seed));
        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            "repo_name".to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );
        // since homeserver_url is not to be provided it will be resolved from the the seed
        // thus needs to be published beforehand
        let publish_net = testnet.bootstrap.clone();
        let _ = publish_url(&key_pair, &homeserver_url, publish_net);

        let mut client = Client::new(Some(testnet.bootstrap));

        assert_eq!(client.homeservers_cache.len(), 0);

        let got_user_id = client.signup(seed, None).unwrap();

        assert_eq!(got_user_id, user_id);
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
        let seed = generate_seed();

        let user_id = get_user_id(Some(&seed));
        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            "repo_name".to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );

        let mut client = Client::new(Some(testnet.bootstrap));

        assert_eq!(client.homeservers_cache.len(), 0);

        client.signup(seed, Some(homeserver_url.clone())).unwrap();

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
        let seed = generate_seed();

        let key_pair = generate_keypair(Some(&seed));
        let user_id = get_user_id(Some(&seed));
        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            "repo_name".to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );
        // since homeserver_url is not to be provided it will be resolved from the the seed
        // thus needs to be published beforehand
        let publish_net = testnet.bootstrap.clone();
        let _ = publish_url(&key_pair, &homeserver_url, publish_net);

        let mut client = Client::new(Some(testnet.bootstrap));

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
        let seed = generate_seed();

        let user_id = get_user_id(Some(&seed));
        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            "repo_name".to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );
        let mut client = Client::new(Some(testnet.bootstrap));

        assert_eq!(client.homeservers_cache.len(), 0);

        client.login(seed, Some(homeserver_url.clone())).unwrap();

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
        let seed = generate_seed();
        let testnet = Testnet::new(10);

        let user_id = get_user_id(Some(&seed));
        let repo_name = "test_repo";

        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            repo_name.to_string(),
            "folder_path".to_string(),
            "data".to_string(),
        );
        let mut client = Client::new(Some(testnet.bootstrap));
        let user_id = client.login(seed, Some(homeserver_url.clone())).unwrap();

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
        let seed = generate_seed();

        let user_id = get_user_id(Some(&seed));

        let repo_name = "test_repo";
        let folder_path = "test_path";
        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            repo_name.to_string(),
            folder_path.to_string(),
            "test_payload".to_string(),
        );
        let mut client = Client::new(Some(testnet.bootstrap));
        let user_id = client.login(seed, Some(homeserver_url.clone())).unwrap();

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

        let seed = generate_seed();
        let user_id = get_user_id(Some(&seed));

        let repo_name = "test_repo";
        let folder_path = "test_path";
        let data = "test_payload";

        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            repo_name.to_string(),
            folder_path.to_string(),
            data.to_string(),
        );
        let mut client = Client::new(Some(testnet.bootstrap));
        let user_id = client.login(seed, Some(homeserver_url.clone())).unwrap();

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

        let seed = generate_seed();
        let user_id = get_user_id(Some(&seed));
        let repo_name = "test_repo";
        let folder_path = "test_path";

        let (mut server, homeserver_url) = create_homeserver_mock(
            user_id.to_string(),
            repo_name.to_string(),
            folder_path.to_string(),
            "data".to_string(),
        );
        let mut client = Client::new(Some(testnet.bootstrap));
        let user_id = client.login(seed, Some(homeserver_url.clone())).unwrap();

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
