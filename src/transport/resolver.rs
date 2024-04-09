use pkarr::{dns, Keypair, PkarrClient, PublicKey, SignedPacket};
use reqwest::Url;
use std::collections::HashMap;

pub struct Resolver<'a> {
    relay_url: Option<&'a Url>,
    // NOTE: Cache is needed mostly for DHT lookups
    // TODO: add suport for different cache strategeies:
    // - read through
    // - read around
    // - read ahead
    // - read behind (current implementation)
    cache: HashMap<String, Url>,
}

impl Resolver<'_> {
    /// Creates a new resolver, if relay_url is None, it will publish to DHT
    pub fn new(relay_url: Option<&Url>) -> Resolver {
        Resolver {
            relay_url,
            cache: HashMap::new(),
        }
    }

    /// Resolves home server url using DHT or relay (with name '_pubky')
    pub fn resolve_homeserver(
        &mut self,
        public_key: &PublicKey,
        relay_url: Option<&Url>,
    ) -> Result<&Url, String> {
        if self.cache.contains_key(&public_key.to_string()) {
            return Ok(self
                .cache
                .get(&public_key.to_string())
                .expect("Failed to get value from cache"));
        }

        let packet = match self.lookup(public_key, relay_url) {
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
                            Some(v) => match self
                                .resolve_homeserver_url(&v.as_str().try_into().unwrap(), relay_url)
                            {
                                Err(e) => return Err(e),
                                Ok(url) => {
                                    let key = public_key.to_string();
                                    let _ = &self.cache.insert(key.clone(), url.clone());

                                    return Ok(self
                                        .cache
                                        .get(&key.clone())
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

    /// Publish record to relay or DHT
    pub fn publish(
        &self,
        key_pair: &Keypair,
        homeserver_url: &Url,
        relay_url: Option<&Url>,
    ) -> Result<(), String> {
        let client = PkarrClient::new();
        let mut packet = dns::Packet::new_reply(0);
        let home = format!("home={}", &key_pair.public_key());
        let home = home.as_str();

        packet.answers.push(dns::ResourceRecord::new(
            dns::Name::new("_pubky").unwrap(),
            dns::CLASS::IN,
            7200,
            dns::rdata::RData::TXT(home.try_into().unwrap()),
        ));

        packet.answers.push(dns::ResourceRecord::new(
            dns::Name::new("@").unwrap(),
            dns::CLASS::IN,
            30,
            dns::rdata::RData::CNAME(dns::Name::new(homeserver_url.as_str()).unwrap().into()),
        ));

        let signed_packet = SignedPacket::from_packet(&key_pair, &packet).unwrap();

        let res = match relay_url {
            Some(relay_url) => client.relay_put(relay_url, &signed_packet),
            None => match &self.relay_url {
                Some(relay_url) => client.relay_put(&relay_url, &signed_packet),
                None => {
                    let _ = client.publish(&signed_packet);
                    Ok(())
                }
            },
        };

        match res {
            Ok(_) => {
                let _ = &self.cache.insert(key_pair.to_z32().clone(), url.clone());
                Ok(())
            },
            Err(_e) => Err("Failed to publish".to_string()),
        }
    }

    /// Resolves home server url using DHT or relay (with name '@')
    fn resolve_homeserver_url(
        &self,
        public_key: &PublicKey,
        relay_url: Option<&Url>,
    ) -> Result<Url, String> {
        let packet = match self.lookup(public_key, relay_url) {
            Err(e) => return Err(e),
            Ok(key) => key,
        };

        let records = packet.resource_records("@");

        for record in records {
            match &record.rdata {
                dns::rdata::RData::CNAME(cname) => {
                    // See https://docs.rs/simple-dns/latest/simple_dns/rdata/struct.CNAME.html#fields
                    return Ok(
                        // Url::parse(format!("https://{}", cname.0.to_string()).as_str()).unwrap(),
                        Url::parse(&cname.0.to_string()).unwrap(),
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
                                // return Ok(Url::parse(format!("http://{k}{v}").as_str()).unwrap())
                                return Ok(Url::parse(format!("{k}{v}").as_str()).unwrap());
                            }
                            // None => return Ok(Url::parse(format!("http://{k}").as_str()).unwrap()),
                            None => return Ok(Url::parse(&k.as_str()).unwrap()),
                        }
                    }
                }
                _ => continue,
            }
        }

        Err("No records found".to_string())
    }

    /// Looks up a public key in the relay or DHT
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
            None => Err("No entry found".to_string()),
            Some(entry) => Ok(entry),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_homeserver_from_dht() {
        let key = Keypair::random();

        let url = Url::parse("https://datastore.example.com").unwrap();

        let mut resolver = Resolver::new(Option::<&Url>::None);
        let _ = resolver.publish(&key, &url, Option::<&Url>::None).unwrap();
        let res = resolver
            .resolve_homeserver(&key.public_key(), Option::<&Url>::None)
            .unwrap();

        assert_eq!(res.to_string(), url.to_string());
    }
}
