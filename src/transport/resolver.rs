use crate::error::DHTError as Error;
use pkarr::{dns, Keypair, PkarrClient, PublicKey, SignedPacket};
use reqwest::Url;
use std::collections::HashMap;

pub struct Resolver {
    // NOTE: Cache is needed mostly for DHT lookups. It will be implemented in pkarr v2
    // So cache could be removed after update
    // TODO: add suport for different cache strategeies:
    // - read through
    // - read around
    // - read ahead
    // - read behind (current implementation)
    cache: HashMap<String, Url>,
    bootstrap: Option<Vec<String>>,
}

impl Resolver {
    /// Creates a new resolver
    pub fn new(bootstrap: Option<Vec<String>>) -> Resolver {
        Resolver {
            cache: HashMap::new(),
            bootstrap,
        }
    }

    /// Resolves home server url using DHT (with name '_pubky')
    pub fn resolve_homeserver(&mut self, public_key: &PublicKey) -> Result<Url, Error> {
        if self.cache.contains_key(&public_key.to_string()) {
            return Ok(self
                .cache
                .get(&public_key.to_string())
                .expect("Failed to get value from cache")
                .clone());
        }

        let packet = match self.lookup(public_key) {
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
                            None => return Err(Error::NoRecordsFound),
                            Some(v) => {
                                match self.resolve_homeserver_url(&v.as_str().try_into().unwrap()) {
                                    Err(e) => return Err(e),
                                    Ok(url) => {
                                        let key = public_key.to_string();
                                        let _ = &self.cache.insert(key.clone(), url.clone());

                                        return Ok(self
                                            .cache
                                            .get(&key.clone())
                                            .expect("Failed to get value from cache")
                                            .clone());
                                    }
                                }
                            }
                        }
                    }
                }
                _ => continue,
            }
        }

        Err(Error::NoRecordsFound)
    }

    /// Publish record to DHT
    pub fn publish(&mut self, key_pair: &Keypair, homeserver_url: &Url) -> Result<(), Error> {
        let client = if self.bootstrap.is_some() {
            let bootstrap = self.bootstrap.clone().unwrap();
            PkarrClient::builder()
                .bootstrap(&bootstrap)
                .build()?
        } else {
            PkarrClient::builder().build()?
        };

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

        let signed_packet = SignedPacket::from_packet(key_pair, &packet)?;

        let res = client.publish(&signed_packet);

        match res {
            Ok(_) => {
                let _ = &self
                    .cache
                    .insert(key_pair.to_z32().clone(), homeserver_url.clone());
                Ok(())
            }
            Err(e) => Err(Error::EntryNotPublished(e.to_string())),
        }
    }

    /// Resolves home server url using DHT (with name '@')
    fn resolve_homeserver_url(&self, public_key: &PublicKey) -> Result<Url, Error> {
        let packet = match self.lookup(public_key) {
            Err(e) => return Err(e),
            Ok(key) => key,
        };

        let records = packet.resource_records("@");

        for record in records {
            match &record.rdata {
                dns::rdata::RData::CNAME(cname) => {
                    // See https://docs.rs/simple-dns/latest/simple_dns/rdata/struct.CNAME.html#fields
                    return Ok(Url::parse(&cname.0.to_string())?);
                }
                dns::rdata::RData::TXT(txt) => {
                    // See https://docs.rs/simple-dns/latest/simple_dns/rdata/struct.TXT.html#method.attributes
                    for (k, v) in txt.attributes() {
                        if !k.starts_with("localhost") {
                            continue;
                        }
                        match v {
                            Some(v) => return Ok(Url::parse(format!("{k}{v}").as_str())?),
                            None => return Ok(Url::parse(k.as_str())?),
                        }
                    }
                }
                _ => continue,
            }
        }

        Err(Error::NoRecordsFound)
    }

    /// Looks up a public key in the DHT
    fn lookup<'a>(&self, public_key: &PublicKey) -> Result<SignedPacket, Error> {
        let client = if self.bootstrap.is_some() {
            let bootstrap = self.bootstrap.clone().unwrap();
            PkarrClient::builder()
                .bootstrap(bootstrap.as_ref())
                .build()?
        } else {
            PkarrClient::builder().build()?
        };

        match client.resolve(public_key) {
            Ok(entry) => match entry {
                Some(entry) => Ok(entry),
                None => Err(Error::EntryNotFound(public_key.clone().to_string())),
            },
            Err(e) => Err(Error::FailedToResolveHomeserverUrl(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_homeserver_from_dht() {
        use mainline::dht::Testnet;
        let testnet = Testnet::new(10);

        let key = Keypair::random();

        let url = Url::parse("https://datastore.example.com").unwrap();

        let mut resolver = Resolver::new(Some(testnet.bootstrap));
        resolver.publish(&key, &url).unwrap();
        let res = resolver.resolve_homeserver(&key.public_key()).unwrap();

        assert_eq!(res.to_string(), url.to_string());
    }
}
