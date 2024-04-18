use crate::transport::{
    auth::SigType,
    challenge::Challenge,
    crypto::Keypair,
    http::{Method, Url},
    resolver::Resolver,
};
use crate::utils::now;

use mainline::dht::Testnet;

pub fn publish_url<'a>(
    key_pair: &'a Keypair,
    url: &'a Url,
    bootstrap: &'a Vec<String>,
) -> Resolver<'a> {
    let mut resolver = Resolver::new(None, Some(bootstrap));
    let _ = resolver.publish(key_pair, url, None).unwrap();

    resolver
}

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

pub fn create_homeserver_mock(
    user_id: String,
    repo_name: String,
    folder_path: String,
    data: String,
) -> mockito::ServerGuard {
    let challenge = Challenge::create(now() + 1000, None);

    // AUTH
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
        headers: vec![("Set-Cookie", "sessionId=send_signature_signup")],
        status: 200,
        body: &b"ok".to_vec(),
    };

    let path = format!("/mvp/session/{}", user_id);
    let send_user_root_signature_login_mock_params = HttpMockParams {
        method: &Method::PUT,
        path: path.as_str(),
        headers: vec![("Set-Cookie", "sessionId=send_signature_login")],
        status: 200,
        body: &b"ok".to_vec(),
    };

    let get_session_mock_params = HttpMockParams {
        method: &Method::GET,
        path: "/mvp/session",
        headers: vec![("Set-Cookie", "sessionId=get_session")],
        body: &b"session".to_vec(), // TODO: proper session object
        status: 200,
    };

    let path = format!("/mvp/session/{}", user_id);
    let logout_mock_params = HttpMockParams {
        method: &Method::DELETE,
        path: path.as_str(),
        status: 200,
        body: &b"ok".to_vec(),
        headers: vec![],
    };

    // REPO
    let path = format!("/mvp/users/{}/repos/{}", user_id, repo_name);
    let create_repo_mock_params = HttpMockParams {
        method: &Method::PUT,
        path: path.as_str(),
        headers: vec![("Set-Cookie", "sessionId=create_repo")],
        status: 200,
        body: &b"ok".to_vec(),
    };

    let path = format!("/mvp/users/{}/repos/{}/{}", user_id, repo_name, folder_path);
    let put_folder_mock_params = HttpMockParams {
        method: &Method::PUT,
        path: path.as_str(),
        headers: vec![("Set-Cookie", "sessionId=create_folder")],
        status: 200,
        body: &b"ok".to_vec(),
    };

    let path = format!("/mvp/users/{}/repos/{}/{}", user_id, repo_name, folder_path);
    let get_data_mock_params = HttpMockParams {
        method: &Method::GET,
        path: path.as_str(),
        headers: vec![("Set-Cookie", "sessionId=get_data")],
        status: 200,
        body: &data.as_bytes().to_vec(),
    };

    let path = format!("/mvp/users/{}/repos/{}/{}", user_id, repo_name, folder_path);
    let delete_data_mock_params = HttpMockParams {
        method: &Method::DELETE,
        path: path.as_str(),
        headers: vec![("Set-Cookie", "sessionId=delete_data")],
        status: 200,
        body: &b"totally ok".to_vec(),
    };

    create_server(vec![
        get_challange_mock_params,
        send_user_root_signature_signup_mock_params,
        send_user_root_signature_login_mock_params,
        get_session_mock_params,
        logout_mock_params,
        create_repo_mock_params,
        put_folder_mock_params,
        get_data_mock_params,
        delete_data_mock_params,
    ])
}
