mod transport;
use crate::transport::pkarr::lookup;
use pkarr::DEFAULT_PKARR_RELAY;

use reqwest::Url;

mod transport_fs;
mod paykit;


fn main() { 
    let res = lookup("hexxxkc4rn3c6hsg17eugwbz88ci9qsa4f87qe85e89jstmmfo5o".try_into().expect("failed key"), "_foo", Option::<&Url>::None);
    println!("DHT: {:?}", res);

    let res = lookup("hexxxkc4rn3c6hsg17eugwbz88ci9qsa4f87qe85e89jstmmfo5o".try_into().expect("failed key"), "_foo", Some(&Url::parse(DEFAULT_PKARR_RELAY).unwrap()));
    println!("Relay: {:?}", res);
}
