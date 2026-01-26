use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to spawn PTY: {0}")]
    PtySpawn(#[from] anyhow::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Password file not found: {0}")]
    PasswordFileNotFound(String),

    #[error("Environment variable not set: {0}")]
    EnvVarNotSet(String),

    #[error("SSH command not found")]
    SshNotFound,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Connection timeout")]
    Timeout,

    #[error("SSH process exited with code: {0}")]
    ExitCode(i32),
}

pub type Result<T> = std::result::Result<T, Error>;
