use crate::{Error, Result};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;

/// Source of the SSH password
#[derive(Debug, Clone)]
pub enum PasswordSource {
    /// Password provided directly as a string
    Plain(String),
    /// Password read from a file (first line)
    File(PathBuf),
    /// Password read from environment variable
    Env(String),
}

impl PasswordSource {
    /// Resolve the password from the source
    pub fn resolve(&self) -> Result<String> {
        match self {
            PasswordSource::Plain(password) => Ok(password.clone()),
            PasswordSource::File(path) => {
                let content = std::fs::read_to_string(path).map_err(|_| {
                    Error::PasswordFileNotFound(path.display().to_string())
                })?;
                Ok(content.lines().next().unwrap_or("").to_string())
            }
            PasswordSource::Env(var) => std::env::var(var)
                .map_err(|_| Error::EnvVarNotSet(var.clone())),
        }
    }
}

/// Builder for SSH connections with password authentication
#[derive(Debug, Clone)]
pub struct SshPassBuilder {
    password_source: PasswordSource,
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    command: Option<Vec<String>>,
    timeout: Duration,
    ssh_options: Vec<String>,
}

impl SshPassBuilder {
    /// Create a new builder with the given password
    pub fn new(password: impl Into<String>) -> Self {
        Self {
            password_source: PasswordSource::Plain(password.into()),
            host: None,
            port: None,
            user: None,
            command: None,
            timeout: Duration::from_secs(30),
            ssh_options: Vec::new(),
        }
    }

    /// Create a new builder with a password source
    pub fn with_source(source: PasswordSource) -> Self {
        Self {
            password_source: source,
            host: None,
            port: None,
            user: None,
            command: None,
            timeout: Duration::from_secs(30),
            ssh_options: Vec::new(),
        }
    }

    /// Set the target host (can include user@ prefix)
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Set the SSH port (default: 22)
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set the username
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Set a command to execute on the remote host
    pub fn command(mut self, args: Vec<String>) -> Self {
        self.command = Some(args);
        self
    }

    /// Set the connection timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add an SSH option (-o key=value)
    pub fn ssh_option(mut self, option: impl Into<String>) -> Self {
        self.ssh_options.push(option.into());
        self
    }

    /// Build and return the SshPass instance
    pub fn build(self) -> Result<SshPass> {
        let host = self.host.ok_or(Error::SshNotFound)?;
        Ok(SshPass {
            password_source: self.password_source,
            host,
            port: self.port,
            user: self.user,
            command: self.command,
            timeout: self.timeout,
            ssh_options: self.ssh_options,
        })
    }

    /// Build and run the SSH connection
    pub fn run(self) -> Result<SshPassOutput> {
        self.build()?.run()
    }
}

/// SSH connection with password authentication
#[derive(Debug, Clone)]
pub struct SshPass {
    password_source: PasswordSource,
    host: String,
    port: Option<u16>,
    user: Option<String>,
    command: Option<Vec<String>>,
    timeout: Duration,
    ssh_options: Vec<String>,
}

impl SshPass {
    /// Create a new SshPass builder with a plain password
    pub fn new(password: impl Into<String>) -> SshPassBuilder {
        SshPassBuilder::new(password)
    }

    /// Create a new SshPass builder with a password from file
    pub fn from_file(path: impl Into<PathBuf>) -> SshPassBuilder {
        SshPassBuilder::with_source(PasswordSource::File(path.into()))
    }

    /// Create a new SshPass builder with a password from environment variable
    pub fn from_env(var: impl Into<String>) -> SshPassBuilder {
        SshPassBuilder::with_source(PasswordSource::Env(var.into()))
    }

    /// Run the SSH connection and return output
    pub fn run(&self) -> Result<SshPassOutput> {
        let password = self.password_source.resolve()?;
        let cmd = self.build_command();

        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave);

        let master = pair.master;
        let mut reader = master.try_clone_reader()?;
        let mut writer = master.take_writer()?;

        let mut output = Vec::new();
        let mut buffer = [0u8; 1024];
        let mut password_sent = false;
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > self.timeout {
                return Err(Error::Timeout);
            }

            // Try to read available data
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = &buffer[..n];
                    output.extend_from_slice(chunk);

                    // Check for password prompt
                    if !password_sent {
                        let text = String::from_utf8_lossy(&output);
                        if Self::is_password_prompt(&text) {
                            // Send password
                            writeln!(writer, "{}", password)?;
                            writer.flush()?;
                            password_sent = true;
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => return Err(Error::Io(e)),
            }
        }

        // Wait for the process to finish
        let status = child.wait()?;

        // Check for authentication failure
        let output_str = String::from_utf8_lossy(&output);
        if output_str.contains("Permission denied")
            || output_str.contains("Authentication failed")
        {
            return Err(Error::AuthenticationFailed);
        }

        let exit_code = status.exit_code() as i32;
        if exit_code != 0 {
            return Err(Error::ExitCode(exit_code));
        }

        Ok(SshPassOutput {
            stdout: output,
            exit_code,
        })
    }

    /// Run SSH interactively (for shell access)
    pub fn run_interactive(&self) -> Result<()> {
        let password = self.password_source.resolve()?;
        let cmd = self.build_command();

        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave);

        let master = pair.master;
        let mut reader = master.try_clone_reader()?;
        let mut writer = master.take_writer()?;

        let mut buffer = [0u8; 1024];
        let mut output_buffer = Vec::new();
        let mut password_sent = false;
        let start = std::time::Instant::now();

        // Phase 1: Handle password prompt
        while !password_sent && start.elapsed() < self.timeout {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = &buffer[..n];
                    output_buffer.extend_from_slice(chunk);

                    // Print to stdout
                    std::io::stdout().write_all(chunk)?;
                    std::io::stdout().flush()?;

                    let text = String::from_utf8_lossy(&output_buffer);
                    if Self::is_password_prompt(&text) {
                        writeln!(writer, "{}", password)?;
                        writer.flush()?;
                        password_sent = true;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(Error::Io(e)),
            }
        }

        if !password_sent {
            return Err(Error::Timeout);
        }

        // Phase 2: Interactive mode - forward stdin/stdout
        let writer_clone = writer;
        let stdin_thread = std::thread::spawn(move || {
            let stdin = std::io::stdin();
            let mut stdin_lock = stdin.lock();
            let mut buf = [0u8; 256];
            let mut writer = writer_clone;
            loop {
                match stdin_lock.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if writer.write_all(&buf[..n]).is_err() {
                            break;
                        }
                        let _ = writer.flush();
                    }
                    Err(_) => break,
                }
            }
        });

        // Read from PTY and write to stdout
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    std::io::stdout().write_all(&buffer[..n])?;
                    std::io::stdout().flush()?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }

        let _ = stdin_thread.join();

        let status = child.wait()?;
        let exit_code = status.exit_code() as i32;
        if exit_code != 0 {
            return Err(Error::ExitCode(exit_code));
        }

        Ok(())
    }

    fn build_command(&self) -> CommandBuilder {
        let mut cmd = CommandBuilder::new("ssh");

        // Disable strict host key checking for automation
        cmd.arg("-o");
        cmd.arg("StrictHostKeyChecking=no");
        cmd.arg("-o");
        cmd.arg("UserKnownHostsFile=/dev/null");

        // Add custom SSH options
        for opt in &self.ssh_options {
            cmd.arg("-o");
            cmd.arg(opt);
        }

        // Add port if specified
        if let Some(port) = self.port {
            cmd.arg("-p");
            cmd.arg(port.to_string());
        }

        // Build host string
        let host_str = if let Some(ref user) = self.user {
            format!("{}@{}", user, self.host)
        } else {
            self.host.clone()
        };
        cmd.arg(&host_str);

        // Add remote command if specified
        if let Some(ref remote_cmd) = self.command {
            for arg in remote_cmd {
                cmd.arg(arg);
            }
        }

        cmd
    }

    fn is_password_prompt(text: &str) -> bool {
        let lower = text.to_lowercase();
        lower.contains("password:")
            || lower.contains("password for")
            || lower.ends_with("password: ")
            || lower.ends_with("'s password:")
    }
}

/// Output from an SSH command execution
#[derive(Debug)]
pub struct SshPassOutput {
    /// Raw stdout/stderr output
    pub stdout: Vec<u8>,
    /// Exit code of the SSH process
    pub exit_code: i32,
}

impl SshPassOutput {
    /// Get output as UTF-8 string (lossy conversion)
    pub fn as_str(&self) -> String {
        String::from_utf8_lossy(&self.stdout).to_string()
    }

    /// Check if the command succeeded (exit code 0)
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_prompt_detection() {
        assert!(SshPass::is_password_prompt("user@host's password:"));
        assert!(SshPass::is_password_prompt("Password:"));
        assert!(SshPass::is_password_prompt("password for user:"));
        assert!(!SshPass::is_password_prompt("Hello world"));
    }

    #[test]
    fn test_password_source_plain() {
        let source = PasswordSource::Plain("secret".to_string());
        assert_eq!(source.resolve().unwrap(), "secret");
    }

    #[test]
    fn test_builder() {
        let sshpass = SshPass::new("password")
            .host("example.com")
            .port(2222)
            .user("admin")
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap();

        assert_eq!(sshpass.host, "example.com");
        assert_eq!(sshpass.port, Some(2222));
        assert_eq!(sshpass.user, Some("admin".to_string()));
    }
}
