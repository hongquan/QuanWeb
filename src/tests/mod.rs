//! Test module for the QuanWeb application
//!
//! This module contains integration tests and test utilities for the application.
//! Tests are organized by feature area.

#[cfg(test)]
pub mod test_files_api;

// Re-export test utilities for use in other test modules
#[cfg(test)]
pub use test_files_api::*;
