mod transport;
use crate::transport::dht::lookup;

mod transport_fs;
mod paykit;


fn main() { 
    lookup("hexxxkc4rn3c6hsg17eugwbz88ci9qsa4f87qe85e89jstmmfo5o".try_into().expect("failed key"));
}
