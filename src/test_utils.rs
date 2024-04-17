use crate::transport::{
    auth::SigType,
    challenge::Challenge,
    crypto::Keypair,
    http::{Method, Url},
    resolver::Resolver,
};
use crate::utils::now;

use mainline::dht::Testnet;

pub struct HttpMockParams<'a> {
    pub method: &'a Method,
    pub path: &'a str,
    pub status: u16,
    pub body: &'a Vec<u8>,
    pub headers: Vec<(&'a str, &'a str)>,
}

pub fn create_server(params: Vec<HttpMockParams>) -> mockito::ServerGuard {
    let mut server = mockito::Server::new();

    for param in params {
        let mut request = server.mock(&param.method.as_str(), param.path);
        request = request.with_status(param.status.into());
        request = request.with_body(param.body);
        for (key, value) in param.headers {
            request = request.with_header(key, value);
        }
        request.create();
    }

    return server;
}

pub fn create_server_for_signup(user_id: String) -> mockito::ServerGuard {
    let challenge = Challenge::create(now() + 1000, None);

    let get_challange_mock_params = HttpMockParams {
        method: &Method::GET,
        path: "/mvp/challenge",
        body: &challenge.serialize(),
        status: 200,
        headers: vec![],
    };

    let path = format!("/mvp/users/{}/pkarr", user_id);
    let send_user_root_signature_signup_mock_params = HttpMockParams {
        method: &Method::PUT,
        path: path.as_str(),
        headers: vec![("Set-Cookie", "sessionId=123")],
        status: 200,
        body: &b"ok".to_vec(),
    };

    create_server(vec![
        get_challange_mock_params,
        send_user_root_signature_signup_mock_params,
    ])
}

pub fn publish_url(key_pair: &Keypair, url: &Url, bootstrap: &Vec<String>) {
    let mut resolver = Resolver::new(None, Some(bootstrap));
    let _ = resolver.publish(key_pair, url, None).unwrap();
}
