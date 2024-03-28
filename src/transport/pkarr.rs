use reqwest::Url;
use pkarr::{PkarrClient, PublicKey, dns };

use crate::transport::pkarr::dns::rdata::RData::TXT;

// Look up:
// take public key as an input and resolve dns record to the data store
//
pub fn lookup(public_key: PublicKey, name: &str, relay_url: Option<&Url>) -> Result<String, String> {
    let client = PkarrClient::new();
    let entry = match relay_url {
        Some(relay_url) => client.relay_get(relay_url, public_key).unwrap(),
        None => client.resolve_most_recent(public_key)
    };
    let mut res = String::new();
    match entry {
        None => return Err("No entry found".to_string()),
        Some(entry) => entry.resource_records(name).for_each(|record| {
            res = if let TXT(txt) = &record.rdata { txt.clone().try_into().unwrap() } else { "".to_string() };
        })
    }

    if res.is_empty() { Err("No TXT record found".to_string()) } else { Ok(res) }
}

// Resolves home server url using relay (with name '_pubky')
// pub fn resolve_homeserver(&self, public_key: &str) -> Result<&str, Error> {}

// Resolves home server url using relay (with name '@')
// pub fn resolve_homeserver_url(&self, public_key: &str) -> Result<&str, Error> {}
