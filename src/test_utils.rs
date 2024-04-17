use crate::transport::{
    auth::SigType,
    challenge::Challenge,
    crypto::Keypair,
    http::{Method, Url},
    resolver::Resolver,
};
use crate::utils::now;

use mainline::dht::Testnet;

pub fn publish_url(key_pair: &Keypair, url: &Url, bootstrap: &Vec<String>) {
    let mut resolver = Resolver::new(None, Some(bootstrap));
    let _ = resolver.publish(key_pair, url, None).unwrap();
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

pub fn create_server_for_repo(user_id: String, repo_name: String) -> mockito::ServerGuard {
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

    let path = format!("/mvp/users/{}/repos/{}", user_id, repo_name);
    let put_folder_mock_params = HttpMockParams {
        method: &Method::PUT,
        path: path.as_str(),
        headers: vec![("Set-Cookie", "sessionId=1234")],
        status: 200,
        body: &b"very ok".to_vec(),
    };

    create_server(vec![
        get_challange_mock_params,
        send_user_root_signature_signup_mock_params,
        put_folder_mock_params,
    ])
}

pub fn create_server_for_data(
    user_id: String,
    repo_name: String,
    folder_path: String,
) -> mockito::ServerGuard {
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

    let repo_name = "test_repo";
    let folder_path = "test_path";
    let path = format!("/mvp/users/{}/repos/{}/{}", user_id, repo_name, folder_path);
    let create_repo_mock_params = HttpMockParams {
        method: &Method::PUT,
        path: path.as_str(),
        headers: vec![("Set-Cookie", "sessionId=1234")],
        status: 200,
        body: &b"very ok".to_vec(),
    };

    create_server(vec![
        get_challange_mock_params,
        send_user_root_signature_signup_mock_params,
        create_repo_mock_params,
    ])
}

pub fn create_server_for_get_data(
    user_id: String,
    repo_name: String,
    folder_path: String,
    body: String,
) -> mockito::ServerGuard {
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

    let repo_name = "test_repo";
    let folder_path = "test_path";
    let path = format!("/mvp/users/{}/repos/{}/{}", user_id, repo_name, folder_path);
    let get_data_mock_params = HttpMockParams {
        method: &Method::GET,
        path: path.as_str(),
        headers: vec![("Set-Cookie", "sessionId=1234")],
        status: 200,
        body: &b"totally ok".to_vec(),
    };

    create_server(vec![
        get_challange_mock_params,
        send_user_root_signature_signup_mock_params,
        get_data_mock_params,
    ])
}
