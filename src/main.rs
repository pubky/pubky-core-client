mod transport;
mod transport_fs;
mod paykit;

fn main() { 
    transport::pkarr_lookup("hexxxkc4rn3c6hsg17eugwbz88ci9qsa4f87qe85e89jstmmfo5o".try_into().expect("failed key"));
}
