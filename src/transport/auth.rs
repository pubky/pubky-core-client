//     /// Create a new account at the config homeserver
//     pub fn signup(&self, seed: &str) -> Result<&str, Error> {}
//
//     /// Login to an account at the config homeserver
//     pub fn login(&self, seed: &str) -> Result<&str, Error> {}
//
//     /// Logout from a specific account at the config homeserver
//     pub fn logout(&self, userId: &str) -> Result<&str, Error> {}
//
//     /// Examine the current session at the config homeserver
//     pub fn session(&self) -> Result<&str, Error> {}
//
//     /// Generate keypair from a seed
//     pub fn keypair(&self, seed: &str) -> Result<&str, Error> {}
//
//     /// Get challenge, sign it and authenticate
//     pub fn send_user_root_signature(&self, sig_type: &str, keypair: &str) -> Result<&str, Error> {}
//
//     /// Get challenge
//     pub fn get_challenge(&self) -> Result<&str, Error> {}
