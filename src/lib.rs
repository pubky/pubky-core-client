pub mod client;
mod transport;

#[cfg(test)]
pub mod test_utils {
    pub struct HttpMockParams<'a> {
        pub method: &'a str,
        pub path: &'a str,
        pub status: u16,
        pub body: &'a str,
        pub headers: Vec<(&'a str, &'a str)>,
    }

    pub fn setup_datastore(params: Vec<HttpMockParams>) -> mockito::ServerGuard {
        use mockito;

        let mut server = mockito::Server::new();

        for param in params {
            let mut request = server.mock(param.method, param.path);
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
