#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Failed to login")]
    FailedToLogin,

    #[error("Failed to logout")]
    FailedToLogout,

    #[error("Failed to retrieve session")]
    FailedToRetrieveSession,

    #[error("Failed to create repository: {0}")]
    FailedToCreateRepository(HTTPError),

    #[error("Failed to store data in repository: {0}")]
    FailedToStoreData(HTTPError),

    #[error("Failed to retrieve data from repository: {0}")]
    FailedToRetrieveData(HTTPError),

    #[error("Failed to delete data from repository: {0}")]
    FailedToDeleteData(HTTPError),
}

// #[derive(thiserror::Error, Debug)]
// pub enum AuthError {
//     #[error("Failed to get challenge: {0}")]
//     FailedToGetChallenge(HTTPError),
//
// }

#[derive(thiserror::Error, Debug)]
pub enum HTTPError {
    #[error("Failed to send HTTP request: {0}")]
    RequestFailed(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ChallengeError {
    #[error("Expired challenge")]
    Expired,

    #[error("Invalid signature")]
    InvalidSignature,
}
// pub enum Error {
//     // #[error("DHT error: {0}")]
//     // pub DHT,
//     //
//     // #[error("HTTP error: {0}")]
//     // pub HTTP,
//     //
//     // #[error("Challenge error: {0}")]
//     // pub Challenge,
//     //
//     // #[error("Auth error: {0}")]
//     // pub Auth
//     //
//     #[error("Client error: {0}")]
//     pub Client
// }

// #[derive(thiserror::Error, Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// enum DHT {
//     #[error("DHT entry not found: {0}")]
//     EntryNotFound(String),
//
//     #[error("Failed to publish DHT entry : {0}")]
//     EntryNotPublished(String),
// }
//
// #[derive(thiserror::Error, Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// enum HTTP {
//     #[error("Failed to send HTTP request: {0}")]
//     RequestFailed(String),
// }
//
// #[derive(thiserror::Error, Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// enum Challenge {
//     #[error("Expired challenge")]
//     Expired,
//
//     #[error("Invalid signature")]
//     InvalidSignature,
// }
//
// #[derive(thiserror::Error, Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// enum Auth {
//     #[error("Invalid signature length: {0}")]
//     InvalidSignatureLength(String),
//
//     #[error("Failed to retrieve session: {0}")]
//     FailedToRetrieveSession(HTTP),
//
//     #[error("Not authenticated with homeserver")]
//     NotAuthenticated,
//
//     #[error("Logout failed: {0}")]
//     LogoutFailed(HTTP),
//
//     #[error("Already logged out")]
//     AlreadyLoggedOut,
// }

