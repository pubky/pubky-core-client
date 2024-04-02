mod transport;
mod crypto;
mod challenge;
use crate::transport::resolver::Resolver;
use crate::transport::http;
use pkarr::DEFAULT_PKARR_RELAY;

use reqwest::Url;

mod paykit;
mod transport_fs;

fn main() {
    let mut session_id = Some("sessionId".to_string());
    let res = http::request(
        reqwest::Method::GET,
        Url::parse(&format!("{}", "http://google.com")).unwrap(),
        &mut session_id,
        None,
        None,
    );

    match res {
        Ok(res) => {
            println!("Success: {:?}", res);
        }
        Err(err) => {
            println!("Error: {:?}", err);
        }
    }
}
