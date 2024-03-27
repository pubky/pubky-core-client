use pkarr::{PkarrClient, PublicKey, dns};
use crate::transport::dht::dns::rdata::{RData, A, AAAA, CNAME, TXT};
use std::net::{Ipv4Addr, Ipv6Addr};

// Look up:
// take public key as an input and resolve dns record to the data store
pub fn lookup(public_key: PublicKey) {
    let client = PkarrClient::new();
    let entry = client.resolve_most_recent(public_key);
    let name = "_foo"; // TODO: use `_pubky` instead
    //
    // TODO: should http call try to query all records or TXT only?
    entry.unwrap().resource_records(&name).for_each(|record| {
        let res = match &record.rdata {
            RData::A(A { address }) => format!("A  {}", Ipv4Addr::from(*address)),
            RData::AAAA(AAAA { address }) => format!("AAAA  {}", Ipv6Addr::from(*address)),
            RData::CNAME(name) => format!("CNAME  {}", name.to_string()),
            RData::TXT(txt) => {
                format!(
                    "TXT  \"{}\"",
                    txt.clone()
                        .try_into()
                        .unwrap_or("__INVALID_TXT_VALUE_".to_string())
                )
            },
            _ => format!("{:?}", record.rdata),
        };

        println!("{}  {:?}", record.name, res);
    });
    // return data in some format ?
}
