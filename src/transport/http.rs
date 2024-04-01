use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use reqwest::Body;
use reqwest::Method;
use reqwest::Url;
use std::collections::HashMap;

// Have a hashmap homeserverUrl -> sessionId
// Q: how to clean it? -> delete manually
//
// IMO it is better for client to handle resolving and for http to handle sessions
// HomeserverUrl + path vs path (including homeserverUrl)
pub fn request(
    method: Method,
    path: Url,
    sessionId: &mut Option<String>,
    headers: Option<&HeaderMap>,
    body: Option<String>,
) -> Result<String, String> {
    // TODO: move somewhere outside?
    let client = Client::new();
    let mut request_builder = client.request(method, path);

    if let Some(body) = body {
        request_builder = request_builder.body(body);
    }

    if let Some(sessionId) = sessionId {
        request_builder = request_builder.header("cookie", format!("sessionId={}", sessionId));
    }

    if let Some(headers) = headers {
        request_builder = request_builder.headers(headers.clone());
    }
    request_builder = request_builder.header("credentials", "include");

    match request_builder.send() {
        Ok(res) => {
            let found_session_id = res.cookies().find(|c| c.name() == "sessionId");
            if let Some(s_id) = found_session_id {
                *sessionId = Some(s_id.value().to_string());
            }
            Ok(res.text().unwrap())
        }
        Err(err) => Err(format!("Error: {:?}", err)),
    }
}
