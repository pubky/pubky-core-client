mod transport;
use crate::transport::resolver::Resolver;
use pkarr::DEFAULT_PKARR_RELAY;

use reqwest::Url;

mod paykit;
mod transport_fs;

fn main() {
    let key = "hexxxkc4rn3c6hsg17eugwbz88ci9qsa4f87qe85e89jstmmfo5o";
    let relay_url = Url::parse(&DEFAULT_PKARR_RELAY).unwrap();

    let mut resolver = Resolver::new(Option::<&Url>::None);
    let res =
        resolver.resolve_homeserver(key.try_into().expect("failed key"), Option::<&Url>::None);
    println!("DHT: {:?}", res);

    let res = resolver.resolve_homeserver(key.try_into().expect("failed key"), Some(&relay_url));
    println!("DHT (relay): {:?}", res);

    let mut resolver = Resolver::new(Some(&relay_url));
    let res =
        resolver.resolve_homeserver(key.try_into().expect("failed key"), Option::<&Url>::None);
    println!("Resolver Relay: {:?}", res);

    let res = resolver.resolve_homeserver(key.try_into().expect("failed key"), Some(&relay_url));
    println!("Method Relay: {:?}", res);
}
