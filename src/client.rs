use std::collections::HashMap;

use crate::transport::{crypto, auth::Auth, resolver::Resolver, http::{Url, Method, request}};

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



pub struct Client {
    seed: [u8; 32],
    homeservers_cache: HashMap<String, (Url, String)>,
    homeserver_url: Url,
}

impl Client<> {
//     pub fn new(homeserverId: &str, config: &ClientConfig) -> Client {
//         let relay = config.relay.unwrap_or("relay.pkarr.org");
//         let mut homeserver_url = config.homeserver_url.unwrap();
//         let mut homeserver_id = homeserver_id;
//         if homeserver_id.starts_with("http") {
//             homeserver_url = homeserver_id;
//         } else {
//             let public_key = PublicKey::from_str(homeserver_id).unwrap();
//             homeserver_url = pkkar_lookup(public_key);
//             homeserver_id = encode(homeserver_id);
//         };
//         // TODO let repos
//
//         Client {
//             session_id: "",
//             // TODO: repos: repos,
//             homeserver_url: homeserver_url,
//             homeserver_id: homeserver_id,
//             relay: relay,
//             homeservers_cache: HashMap::new(),
//         }
//     }
    pub fn new(seed: Option<[u8; 32]>, homeserver_url: Option<Url>, dht_relay: Option<Url>, bootstrap: Option<& Vec<String>>) -> Client {
        let seed = seed.unwrap_or(crypto::random_bytes(32).try_into().unwrap());

        let resolver = Resolver::new(dht_relay.as_ref(), bootstrap);
        let mut auth = Auth::new(resolver, homeserver_url);

        let user_id = auth.signup(&seed, None).unwrap();
        let homeserver_url = auth.homeserver_url.unwrap();
        let session_id = auth.session_id.unwrap();

        let mut homeservers_cache = HashMap::new();
        homeservers_cache.insert(user_id, (homeserver_url.clone(), session_id));

        Client {
            seed,
            homeservers_cache,
            homeserver_url,
        }
    }

    /* "REPOS" RELATED LOGIC */

    /// Create repository for user
    pub fn create(&mut self, user_id: &str, repo_name: &str) -> Result<(), String> {
        let url = &self.homeservers_cache.get(user_id).unwrap().0;
        let url = url.join(&format!("/mvp/users/{}/repos/{}", user_id, repo_name)).unwrap();

        let mut session_id = &self.homeservers_cache.get_mut(user_id).unwrap().1;
        // XXX: make sure that it will update session in the cache preferably by ref
        match request(Method::PUT, url.clone(), &mut Some(session_id.to_string()), None, None) {
            Ok(_) => Ok(()),
            Err(e) => return Err(e),
        }
    }

//     // get, delete, list, query
//     /// Put data into user's repository and return URL to this repo
//     pub fn put (&self, user_id:&str, repo_name: &str, path: &str, payload: &str) -> Result<Url, String> {
//         Ok()
//     }
//
//     /// Get data from user's repository and return it as a JSON(?)
//     pub fn get (&self, user_id: &str, repo_name: &str, path: &str) -> Result<Data, String> { }
//
//     /// Delete data from user's repository
//     pub fn delete (&self, user_id: &str, repo_name: &str, path: &str) -> Result<_, String> { }
//
//     /// List data in user's repository
//     /*
//     ListOption {
//          reverse: bool,
//          offset: usize,
//          limit: usize,
//     }
//     */
//     pub fn list (&self, user_id: &str, repo_name: &str, path: &str, opts: Option<ListOption>) -> Result<Vec<String>, String> { }
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
//     pub fn query (&self, user_id: &str, repo_name: &str, query: Option<QueryOptions>) -> Result<Vec<String>, String> { }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{setup_datastore, HttpMockParams};
    use crate::transport::crypto::{Keypair, DeterministicKeyGen};
    use crate::transport::challenge::Challenge;
    use mainline::dht::Testnet;
    use std::time::{SystemTime, UNIX_EPOCH};

    // TODO: move to helper
    fn now() -> u64 {
        let now = SystemTime::now();
        now.duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }


    #[test]
    fn test_client_new() {
        let testnet = Testnet::new(10);
        let challenge = Challenge::create(now() + 1000, None);

        let seed = b"it is a seed for key generation!";
        let key_pair: Keypair = DeterministicKeyGen::generate(Some(seed));
        let user_id = key_pair.to_z32();

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

        let server = setup_datastore(vec![
            get_challange_mock_params,
            send_user_root_signature_signup_mock_params,
        ]);


        let mut resolver = Resolver::new(None, Some(&testnet.bootstrap));
        let _ = resolver
            .publish(&key_pair, &Url::parse(&server.url()).unwrap(), None)
            .unwrap();

        let client = Client::new(Some(seed.clone()), None, None, Some(&testnet.bootstrap));

        assert_eq!(client.homeservers_cache.len(), 1);
        assert_eq!(client.homeservers_cache.get(&user_id).unwrap().0, Url::parse(&server.url()).unwrap());
        assert_eq!(client.homeservers_cache.get(&user_id).unwrap().1.clone(), "123".to_string());
    }

    #[test]
    fn test_client_create() {
        let testnet = Testnet::new(10);

        let challenge = Challenge::create(now() + 1000, None);

        let seed = b"it is a seed for key generation!";
        let key_pair: Keypair = DeterministicKeyGen::generate(Some(seed));
        let user_id = key_pair.to_z32();

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

        let repo_name = "test_repo";
        let path = format!("/mvp/users/{}/repos/{}", user_id, repo_name);
        let create_repo_mock_params = HttpMockParams {
            method: &Method::PUT,
            path: path.as_str(),
            headers: vec![("Set-Cookie", "sessionId=1234")],
            status: 200,
            body: &b"very ok".to_vec(),
        };

        let server = setup_datastore(vec![
            get_challange_mock_params,
            send_user_root_signature_signup_mock_params,
            create_repo_mock_params,
        ]);


        let mut resolver = Resolver::new(None, Some(&testnet.bootstrap));
        let _ = resolver
            .publish(&key_pair, &Url::parse(&server.url()).unwrap(), None)
            .unwrap();

        let mut client = Client::new(Some(seed.clone()), None, None, Some(&testnet.bootstrap));

        let result = client.create(&user_id, repo_name);

        assert_eq!(result, Ok(()));
        assert_eq!(client.homeservers_cache.len(), 1);
        assert_eq!(client.homeservers_cache.get(&user_id).unwrap().0, Url::parse(&server.url()).unwrap());
        assert_eq!(client.homeservers_cache.get(&user_id).unwrap().1.clone(), "1234".to_string());
    }
}
