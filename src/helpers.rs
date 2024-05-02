pub struct Path {}

impl Path {
    pub fn get_challenge_string() -> String {
        "/mvp/challenge".to_string()
    }

    pub fn get_session_string(user_id: Option<&str>) -> String {
        match user_id {
            Some(user_id) => format!("/mvp/session/{}", user_id),
            None => "/mvp/session".to_string(),
        }
    }

    pub fn get_signup_string(user_id: &str) -> String {
        format!("/mvp/users/{}/pkarr", user_id)
    }

    pub fn get_repo_string(user_id: &str, repo_name: &str, path: Option<&str>) -> String {
        match path {
            Some(path) => {
                if path.starts_with("/") {
                    format!("/mvp/users/{}/repos/{}/{}", user_id, repo_name, &path[1..])
                } else {
                    format!("/mvp/users/{}/repos/{}/{}", user_id, repo_name, path)
                }
            }
            None => format!("/mvp/users/{}/repos/{}", user_id, repo_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(Path::get_challenge_string(), "/mvp/challenge");
        assert_eq!(Path::get_session_string(None), "/mvp/session");
        assert_eq!(
            Path::get_session_string(Some("user_id")),
            "/mvp/session/user_id"
        );
        assert_eq!(
            Path::get_signup_string("user_id"),
            "/mvp/users/user_id/pkarr"
        );
        assert_eq!(
            Path::get_repo_string("user_id", "repo_name", None),
            "/mvp/users/user_id/repos/repo_name"
        );
        assert_eq!(
            Path::get_repo_string("user_id", "repo_name", Some("path")),
            "/mvp/users/user_id/repos/repo_name/path"
        );
        assert_eq!(
            Path::get_repo_string("user_id", "repo_name", Some("/path")),
            "/mvp/users/user_id/repos/repo_name/path"
        );
    }
}
