
// Public method that takes a public_key (and decryption key?) and returns the data from the data store
// 1. resolve DNS record by public key using pkarr
// 2. add path (conventional) and do get request to get index
// 3. query each endpoint from index to get payment data
// 4. return data as KV store
//
// Public method that stores data in the data store
//
// Public method that udpates data in the data stores
//
// Public method that deletes data in the data store

// query data store via REST api with specific path to get the index data (authenticated in case of

// private)
// query data store via REST api with specific path to get the data (authenticated in case of private)
//
//
// Storage:
// authenticated REST api call to store file at specific location
//
//
// Helper method to request data from the data Storage by api_base url

pub mod resolver;
pub mod http;
// pub mod client;
