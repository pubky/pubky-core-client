// use std::collections::HashMap;
// use z32::encode;
//
// struct Client {
//   session_id: &str,
//   homeserver_url: &str,
//   homeserver_id: &str,
//   relay: &str,
//   homeservers_cache: HashMap<String, String>,
// }
//
// struct ClientConfig {
//   homeserver_url: Option<String>,
//   relay: Option<String>,
//   relay: Option<String>,
// }
//
// impl Client {
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
//
//     /* "repos" related logic */
//     /// Create repository for user
//     pub fn create(&self, user_id:&str, repo_name: &str) -> Result<_, String> {
//
//         Ok()
//     };
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
// }
