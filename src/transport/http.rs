use reqwest::blocking::Client;
pub use reqwest::header::HeaderMap;
pub use reqwest::Method;
pub use reqwest::Url;

// Have a hashmap homeserverUrl -> session_id
// Q: how to clean it? -> delete manually
pub fn request(
    method: Method,
    path: Url,
    session_id: &mut Option<String>,
    headers: Option<&HeaderMap>,
    body: Option<String>,
) -> Result<String, String> {
    // TODO: consider moving somewhere outside?
    let client = Client::new();
    let mut request_builder = client.request(method, path);

    if let Some(body) = body {
        request_builder = request_builder.body(body);
    }

    if let Some(session_id) = session_id {
        request_builder = request_builder.header("cookie", format!("sessionId={}", session_id));
    }

    if let Some(headers) = headers {
        request_builder = request_builder.headers(headers.clone());
    }
    request_builder = request_builder.header("credentials", "include");

    match request_builder.send() {
        Ok(res) => {
            let found_session_id = res.cookies().find(|c| c.name() == "sessionId");
            if let Some(s_id) = found_session_id {
                *session_id = Some(s_id.value().to_string());
            }
            Ok(res.text().unwrap())
        }
        Err(err) => Err(format!("Error: {:?}", err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;

    #[test]
    fn test_request() {
        let server = test_utils::setup_datastore(vec![test_utils::HttpMockParams {
            method: "GET",
            path: "/test",
            status: 200,
            body: "test",
            headers: vec![("Set-Cookie", "sessionId=123")],
        }]);

        let mut session_id = None;
        let headers = HeaderMap::new();
        let body = None;
        let path = Url::parse(&format!("{}/test", server.url())).unwrap();

        let res = request(Method::GET, path, &mut session_id, Some(&headers), body);

        assert_eq!(res.is_ok(), true);
        assert_eq!(session_id.is_some(), true);
        assert_eq!(session_id.unwrap(), "123");
    }
}
