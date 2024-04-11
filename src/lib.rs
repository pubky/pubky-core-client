pub mod client;
mod transport;

#[cfg(test)]
pub mod test_utils {
    use crate::transport::http;

    pub struct HttpMockParams<'a> {
        pub method: &'a http::Method,
        pub path: &'a str,
        pub status: u16,
        pub body: &'a Vec<u8>,
        pub headers: Vec<(&'a str, &'a str)>,
    }

    pub fn setup_datastore(params: Vec<HttpMockParams>) -> mockito::ServerGuard {
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
}
