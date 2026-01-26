//! sshpass - Non-interactive SSH password authentication
//!
//! This crate provides functionality similar to the Unix `sshpass` utility,
//! allowing automated SSH connections with password authentication.
//!
//! # Example
//!
//! ```no_run
//! use sshpass::{SshPass, PasswordSource};
//!
//! let result = SshPass::new("mypassword")
//!     .host("user@example.com")
//!     .run();
//! ```

mod error;
mod sshpass;

pub use error::{Error, Result};
pub use sshpass::{PasswordSource, SshPass, SshPassBuilder};
