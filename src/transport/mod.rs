use pkarr::{PkarrClient, PublicKey, dns};
use crate::transport::dns::rdata::RData;
use crate::transport::dns::rdata::A;
use crate::transport::dns::rdata::AAAA;
use crate::transport::dns::rdata::CNAME;
use crate::transport::dns::rdata::TXT;

use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

// Quesitons:
// - how encrypted data is handled
// - how path is handled (what a the conventions for storing specific data)

/// Public method that takes a public_key (and decryption key?) and returns the data from the data store
/// 1. resolve DNS record by public key using pkarr
/// 2. add path (conventional) and do get request to get index
/// 3. query each endpoint from index to get payment data
/// 4. return data as KV store
///
/// Public method that stores data in the data store
///
/// Public method that udpates data in the data stores
///
/// Public method that deletes data in the data store

// Look up:
// take public key as an input and resolve dns record to the data store
pub fn pkarr_lookup(public_key: PublicKey) {
    let client = PkarrClient::new();
    // What is the type of entry
    let entry = client.resolve_most_recent(public_key);
    // TODO: use `_pubky` instead
    let name = "_foo";
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

    // let data = client.get_data(public_key, index);

    // TODO:
    // check if public key matches?
    // check if signature is valid?
    // get packet and extract data
    // return data
}
// query data store via REST api with specific path to get the index data (authenticated in case of

// private)
// query data store via REST api with specific path to get the data (authenticated in case of private)
//
//
// Storage:
// authenticated REST api call to store file at specific location
//
//
// Helper method to request data from the data Storage by api_base url
