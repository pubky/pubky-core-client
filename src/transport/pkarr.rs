use reqwest::Url;
use pkarr::{PkarrClient, PublicKey, dns, SignedPacket};

pub fn lookup<'a>(public_key: PublicKey, relay_url: Option<&'a Url>) -> Result<SignedPacket, String> {
    let client = PkarrClient::new();
    let entry = match relay_url {
        Some(relay_url) => client.relay_get(relay_url, public_key).unwrap(),
        None => client.resolve_most_recent(public_key)
    };

    match entry {
        None => return Err("No entry found".to_string()),
        Some(entry) => Ok(entry)
    }
}

/// Resolves home server url using relay (with name '_pubky')
pub fn resolve_homeserver(public_key: PublicKey, relay_url: Option<&Url>) -> Result<Url, String> {
    // TODO: check cache and return if found

    let packet = match lookup(public_key, relay_url) {
        Err(e) => return Err(e),
        Ok(key) => key
    };

    let records = packet.resource_records("_pubky");
    for record in records {
        match &record.rdata {
            dns::rdata::RData::TXT(txt) => {
                // See https://docs.rs/simple-dns/latest/simple_dns/rdata/struct.TXT.html#method.attributes
                for (k, v) in txt.attributes() {
                    if !k.starts_with("home") { continue; }

                    // TODO: make sure that `attributes` returned correct values
                    match v {
                        None => return Err("No value found".to_string()),
                        Some(v) => match resolve_homeserver_url(v.as_str().try_into().expect("failed key"), relay_url) {
                            Err(e) => return Err(e),
                            Ok(url) => {
                                // TODO: set to cache
                                return Ok(url)
                            }
                        }
                    }
                }
            },
            _ => continue,
        }
    }

    Err("No records found".to_string())
}

/// Resolves home server url using relay (with name '@')
pub fn resolve_homeserver_url(public_key: PublicKey, relay_url: Option<&Url>) -> Result<Url, String> {
    let packet = match lookup(public_key, relay_url) {
        Err(e) => return Err(e),
        Ok(key) => key
    };

    let records = packet.resource_records("@");

    for record in records {
        match &record.rdata {
            dns::rdata::RData::CNAME(cname) => {
                // See https://docs.rs/simple-dns/latest/simple_dns/rdata/struct.CNAME.html#fields
                return Ok(Url::parse(format!("https://{}", cname.0.to_string()).as_str()).unwrap())
            },
            dns::rdata::RData::TXT(txt) => {
                // See https://docs.rs/simple-dns/latest/simple_dns/rdata/struct.TXT.html#method.attributes
                for (k, v) in txt.attributes() {
                    if !k.starts_with("localhost") { continue; }
                    match v {
                        Some(v) => return Ok(Url::parse(format!("http://{k}{v}").as_str()).unwrap()),
                        None => return Ok(Url::parse(format!("http://{k}").as_str()).unwrap())
                    }
                }
            },
            _ => continue,
        }
    }

    Err("No records found".to_string())
}
