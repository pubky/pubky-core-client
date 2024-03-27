use std::collections::HashMap;
use z32::encode;

struct Client {
  sessionId: &str,
  homeserverUrl: &str,
  homeserverId: &str,
  relay: &str,
  homeserversCache: HashMap<String, String>,
}

struct ClientConfig {
  homeserverUrl: Option<String>,
  relay: Option<String>,
}

impl Client {
    pub fn new(homeserverId: &str, config: &ClientConfig) -> Client {
        let relay = config.relay.unwrap_or("relay.pkarr.org");
        let mut homeserverUrl = config.homeserverUrl.unwrap();
        let mut homeserverId = homeserverId;
        // let repos
        if homeserverId.starts_with("http") {
            homeserverUrl = homeserverId;
        } else {
            let public_key = PublicKey::from_str(homeServerId).unwrap();
            homeserverUrl = pkkar_lookup(public_key);
            homeserverId = encode(homeServerId);
        };

        Client {
            sessionId: "",
            homeserverUrl: homeserverUrl,
            homeserverId: homeserverId,
            relay: relay,
            homeserversCache: HashMap::new(),
        }
    }
}
