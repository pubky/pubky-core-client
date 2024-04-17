use crate::transport::{
    auth::SigType,
    http::{Method, Url},
    resolver::Resolver,
};
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
