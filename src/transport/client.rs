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
//     /// Create a new account at the config homeserver
//     pub fn signup(&self, seed: &str) -> Result<&str, Error> {}
//
//     /// Login to an account at the config homeserver
//     pub fn login(&self, seed: &str) -> Result<&str, Error> {}
//
//     /// Logout from a specific account at the config homeserver
//     pub fn logout(&self, userId: &str) -> Result<&str, Error> {}
//
//     /// Examine the current session at the config homeserver
//     pub fn session(&self) -> Result<&str, Error> {}
//
//     /// Generate keypair from a seed
//     pub fn keypair(&self, seed: &str) -> Result<&str, Error> {}
//
//     /// Internal helper to parse
//     fn parse_id(id: &str) -> Result<&str, Error> {}
// }
