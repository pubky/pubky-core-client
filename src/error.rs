use pkarr::Error as PkarrError;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Failed to signup: {0}")]
    FailedToSignup(AuthError),

    #[error("Failed to login: {0}")]
    FailedToLogin(AuthError),

    #[error("Failed to logout: {0}")]
    FailedToLogout(AuthError),

    #[error("Failed to retrieve session: {0}")]
    FailedToRetrieveSession(AuthError),

    #[error("Failed to create repository: {0}")]
    FailedToCreateRepository(HTTPError),

    #[error("Failed to store data in repository: {0}")]
    FailedToStoreData(HTTPError),

    #[error("Failed to retrieve data from repository: {0}")]
    FailedToRetrieveData(HTTPError),

    #[error("Failed to delete data from repository: {0}")]
    FailedToDeleteData(HTTPError),

    #[error("User was not signed up")]
    UserNotSignedUp,

    #[error("Provided input parameters can not be converrted to a valid URL")]
    InvalidInputForUrl,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Failed to get challenge: {0}")]
    FailedToGetChallenge(HTTPError),

    #[error("Faield to send user signature: {0}")]
    FailedToSendUserSignature(HTTPError),

    #[error("Failed to resolve homeserver: {0}")]
    FailedToResolveHomeserver(DHTError),

    #[error("Failed to publish homeserver: {0}")]
    FailedToPublishHomeserver(DHTError),

    #[error("Failed to retrieve session: {0}")]
    FailedToRetrieveSession(HTTPError),

    #[error("No associated homeserver")]
    NoHomeserver,

    #[error("No associated session")]
    NoSession,

    #[error("Failed to logout: {0}")]
    FailedToLogout(HTTPError),
}

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

#[derive(thiserror::Error, Debug, Clone)]
pub enum DHTError {
    #[error("DHT entry not found: {0}")]
    EntryNotFound(String),

    #[error("Failed to publish DHT entry : {0}")]
    EntryNotPublished(String),

    #[error("No records found")]
    NoRecordsFound,

    #[error("Pkarr error: {0}")]
    PkarrError(String),
}

impl From<PkarrError> for DHTError {
    fn from(error: PkarrError) -> Self {
        DHTError::PkarrError(format!("Failed to instantiate Pkarr client: {}", error))
    }
}
