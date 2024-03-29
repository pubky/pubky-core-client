use pkarr::{dns, PkarrClient, PublicKey, SignedPacket};
use reqwest::Url;
use std::collections::HashMap;

pub struct Resolver<'a> {
    relay_url: Option<&'a Url>,
    // TODO: add suport for different cache strategeies:
    // - read through
    // - read around
    // - read ahead
    // - read behind (current implementation)
    cache: HashMap<String, Url>,
}

impl Resolver<'_> {
    pub fn new(relay_url: Option<&Url>) -> Resolver {
        Resolver {
            relay_url,
            cache: HashMap::new(),
        }
    }
    /// Resolves home server url using relay (with name '_pubky')
    pub fn resolve_homeserver(
        &mut self,
        public_key: PublicKey,
        relay_url: Option<&Url>,
    ) -> Result<&Url, String> {
        if self.cache.contains_key(&public_key.to_string()) {
            return Ok(self
                .cache
                .get(&public_key.to_string())
                .expect("Failed to get value from cache"));
        }

        let packet = match self.lookup(&public_key, relay_url) {
            Err(e) => return Err(e),
            Ok(key) => key,
        };
        let records = packet.resource_records("_pubky");

        for record in records {
            match &record.rdata {
                dns::rdata::RData::TXT(txt) => {
                    // See https://docs.rs/simple-dns/latest/simple_dns/rdata/struct.TXT.html#method.attributes
                    for (k, v) in txt.attributes() {
                        if !k.starts_with("home") {
                            continue;
                        }

                        match v {
                            None => return Err("No value found".to_string()),
                            Some(v) => match self.resolve_homeserver_url(
                                v.as_str().try_into().expect("failed key"),
                                relay_url,
                            ) {
                                Err(e) => return Err(e),
                                Ok(url) => {
                                    let _ = &self.cache.insert(public_key.to_string(), url.clone());

                                    return Ok(self
                                        .cache
                                        .get(&public_key.to_string())
                                        .expect("Failed to get value from cache"));
                                }
                            },
                        }
                    }
                }
                _ => continue,
            }
        }

        Err("No records found".to_string())
    }

    /// Resolves home server url using relay (with name '@')
    fn resolve_homeserver_url(
        &self,
        public_key: PublicKey,
        relay_url: Option<&Url>,
    ) -> Result<Url, String> {
        let packet = match self.lookup(&public_key, relay_url) {
            Err(e) => return Err(e),
            Ok(key) => key,
        };

        let records = packet.resource_records("@");

        for record in records {
            match &record.rdata {
                dns::rdata::RData::CNAME(cname) => {
                    // See https://docs.rs/simple-dns/latest/simple_dns/rdata/struct.CNAME.html#fields
                    return Ok(
                        Url::parse(format!("https://{}", cname.0.to_string()).as_str()).unwrap(),
                    );
                }
                dns::rdata::RData::TXT(txt) => {
                    // See https://docs.rs/simple-dns/latest/simple_dns/rdata/struct.TXT.html#method.attributes
                    for (k, v) in txt.attributes() {
                        if !k.starts_with("localhost") {
                            continue;
                        }
                        match v {
                            Some(v) => {
                                return Ok(Url::parse(format!("http://{k}{v}").as_str()).unwrap())
                            }
                            None => return Ok(Url::parse(format!("http://{k}").as_str()).unwrap()),
                        }
                    }
                }
                _ => continue,
            }
        }

        Err("No records found".to_string())
    }

    fn lookup<'a>(
        &self,
        public_key: &PublicKey,
        relay_url: Option<&Url>,
    ) -> Result<SignedPacket, String> {
        let client = PkarrClient::new();
        let public_key = public_key.clone();
        let entry = match relay_url {
            Some(relay_url) => client.relay_get(relay_url, public_key).unwrap(),
            None => match &self.relay_url {
                Some(relay_url) => client.relay_get(&relay_url, public_key).unwrap(),
                None => client.resolve_most_recent(public_key),
            },
        };

        match entry {
            None => return Err("No entry found".to_string()),
            Some(entry) => Ok(entry),
        }
    }
}
