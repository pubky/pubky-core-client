mod transport;
use crate::transport::pkarr::{lookup, resolve_homeserver_url};
use pkarr::DEFAULT_PKARR_RELAY;

use reqwest::Url;

mod transport_fs;
mod paykit;


fn main() { 
    let res = resolve_homeserver_url("hexxxkc4rn3c6hsg17eugwbz88ci9qsa4f87qe85e89jstmmfo5o".try_into().expect("failed key"), Option::<&Url>::None);
    println!("DHT: {:?}", res);

    let relay_url = Url::parse(DEFAULT_PKARR_RELAY).unwrap();
    let res = resolve_homeserver_url("hexxxkc4rn3c6hsg17eugwbz88ci9qsa4f87qe85e89jstmmfo5o".try_into().expect("failed key"), Some(&relay_url));
    println!("Relay: {:?}", res);
}
