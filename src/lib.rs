//! Welcome to the Pubkey Core Rust Library.

/// Main entry point for the library which enables the user to interact with the pubky core network.
pub mod client;

/// Module containing the core errors that can be returned by the library.
pub mod error;
mod helpers;
mod transport;

/// Public utility functions that can be used to interact with the network or library.
pub mod utils;

#[cfg(test)]
mod test_utils;
