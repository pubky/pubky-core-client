use pkarr::Error as PkarrError;
use url::ParseError as UrlParseError;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    /// Error when trying to create a new client on the homeserver
    #[error("Failed to signup: {0}")]
    FailedToSignup(AuthError),

    /// Error when trying to login to the homeserver
    #[error("Failed to login: {0}")]
    FailedToLogin(AuthError),

    /// Error when trying to logout from the homeserver
    #[error("Failed to logout: {0}")]
    FailedToLogout(AuthError),

    /// Error when trying to retrieve a session from the homeserver
    #[error("Failed to retrieve session: {0}")]
    FailedToRetrieveSession(AuthError),

    /// Error when trying to create a new repository
    #[error("Failed to create repository: {0}")]
    FailedToCreateRepository(HTTPError),

    /// Error when trying to store data in a repository
    #[error("Failed to store data in repository: {0}")]
    FailedToStoreData(HTTPError),

    /// Error when trying to retrieve data from a repository
    #[error("Failed to retrieve data from repository: {0}")]
    FailedToRetrieveData(HTTPError),

    /// Error when trying to delete data from a repository
    #[error("Failed to delete data from repository: {0}")]
    FailedToDeleteData(HTTPError),

    /// Error when trying to access data for a user who is not signed up
    #[error("User was not signed up")]
    UserNotSignedUp,

    /// Error when trying to process user input related to generating URLs
    #[error("Provided input parameters can not be converrted to a valid URL")]
    InvalidInputForUrl,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    /// Error when trying to get a challenge from the homeserver
    #[error("Failed to get challenge: {0}")]
    FailedToGetChallenge(HTTPError),

    /// Error when trying to send a user signature to the homeserver
    #[error("Faield to send user signature: {0}")]
    FailedToSendUserSignature(HTTPError),

    /// Error when trying to resolve the homeserver
    #[error("Failed to resolve homeserver: {0}")]
    FailedToResolveHomeserver(DHTError),

    /// Error when trying to publish the homeserver
    #[error("Failed to publish homeserver: {0}")]
    FailedToPublishHomeserver(DHTError),

    /// Error when trying to retrieve session from the homeserver
    #[error("Failed to retrieve session: {0}")]
    FailedToRetrieveSession(HTTPError),

    /// Error when trying to lookup homeserver
    #[error("No associated homeserver")]
    NoHomeserver,

    /// Error when trying to lookup session
    #[error("No associated session")]
    NoSession,

    /// Error when trying to logout
    #[error("Failed to logout: {0}")]
    FailedToLogout(HTTPError),
}

#[derive(thiserror::Error, Debug)]
pub enum HTTPError {
    /// Error when trying to send an HTTP request
    #[error("Failed to send HTTP request: {0}")]
    RequestFailed(String),

    /// Error when trying to parse an HTTP response
    #[error("Failed to parse HTTP response")]
    ResponseParseFailed,
}

#[derive(thiserror::Error, Debug)]
pub enum ChallengeError {
    /// Error when trying to parse a challenge
    #[error("Expired challenge")]
    Expired,

    /// Error when trying to validate a signature
    #[error("Invalid signature")]
    InvalidSignature,
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum DHTError {
    /// Error when trying to resolve a DHT entry
    #[error("DHT entry not found: {0}")]
    EntryNotFound(String),

    /// Error when trying to publish a DHT entry
    #[error("Failed to publish DHT entry : {0}")]
    EntryNotPublished(String),

    /// Error when trying to find record on DHT
    #[error("No records found")]
    NoRecordsFound,

    /// Pkarr error
    #[error("Pkarr error: {0}")]
    PkarrError(String),

    /// Erorr when trying to resolve homeserver URL
    #[error("Failed to resolve homeserver URL: {0}")]
    FailedToResolveHomeserverUrl(String),

    /// Error when trying to parse DNS record as URL
    #[error("Failed to parse DNS record as URL")]
    FailedToParseDnsRecordAsUrl,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum PathError {
    /// Error when trying to generate a URL with invalid path
    #[error("Invalid path")]
    InvalidPath,
}

impl From<PkarrError> for DHTError {
    fn from(e: PkarrError) -> Self {
        DHTError::PkarrError(format!("Failed to instantiate Pkarr client: {}", e))
    }
}

impl From<UrlParseError> for DHTError {
    fn from(_: UrlParseError) -> Self {
        DHTError::FailedToParseDnsRecordAsUrl
    }
}

impl From<PathError> for ClientError {
    fn from(_: PathError) -> Self {
        ClientError::InvalidInputForUrl
    }
}
