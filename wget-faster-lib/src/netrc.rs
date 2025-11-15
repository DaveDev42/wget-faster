/// .netrc file parser for automatic authentication
///
/// The .netrc file format is used by wget and curl for storing authentication credentials.
/// Format:
/// ```text
/// machine example.com
/// login myuser
/// password mypass
/// ```
///
/// Default location: `~/.netrc` on Unix-like systems, `~/_netrc` on Windows
use crate::{AuthConfig, AuthType, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Entry in .netrc file
#[derive(Debug, Clone)]
pub struct NetrcEntry {
    /// Machine hostname (e.g., "example.com")
    pub machine: String,
    /// Login username
    pub login: String,
    /// Login password
    pub password: String,
}

/// .netrc file parser
#[derive(Debug, Clone)]
pub struct Netrc {
    entries: HashMap<String, NetrcEntry>,
    default: Option<NetrcEntry>,
}

impl Netrc {
    /// Parse .netrc file from path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to .netrc file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_string(&content)
    }

    /// Parse .netrc file from default location
    ///
    /// Looks for `~/.netrc` on Unix-like systems, `~/_netrc` on Windows
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    /// Returns Ok(None) if the file doesn't exist.
    pub fn from_default_location() -> Result<Option<Self>> {
        if let Some(path) = Self::default_path() {
            if path.exists() {
                return Ok(Some(Self::from_file(&path)?));
            }
        }
        Ok(None)
    }

    /// Get default .netrc file path
    ///
    /// Returns `~/.netrc` on Unix-like systems, `~/_netrc` on Windows
    pub fn default_path() -> Option<PathBuf> {
        if let Some(home) = std::env::var_os("HOME") {
            let mut path = PathBuf::from(home);
            path.push(if cfg!(windows) { "_netrc" } else { ".netrc" });
            return Some(path);
        }
        None
    }

    /// Parse .netrc content from string
    ///
    /// # Arguments
    ///
    /// * `content` - Content of .netrc file
    ///
    /// # Errors
    ///
    /// Returns an error if the content cannot be parsed
    pub fn from_string(content: &str) -> Result<Self> {
        let mut entries = HashMap::new();
        let mut default = None;

        let mut tokens: Vec<String> = content
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(std::string::ToString::to_string)
            .collect();

        // Remove comments (lines starting with #)
        tokens.retain(|t| !t.starts_with('#'));

        let mut i = 0;
        while i < tokens.len() {
            match tokens[i].as_str() {
                "machine" => {
                    if i + 1 >= tokens.len() {
                        break;
                    }
                    let machine = tokens[i + 1].clone();
                    i += 2;

                    // Parse login and password
                    let mut login = String::new();
                    let mut password = String::new();

                    while i < tokens.len() && tokens[i] != "machine" && tokens[i] != "default" {
                        match tokens[i].as_str() {
                            "login" => {
                                if i + 1 < tokens.len() {
                                    login = tokens[i + 1].clone();
                                    i += 2;
                                } else {
                                    i += 1;
                                }
                            },
                            "password" => {
                                if i + 1 < tokens.len() {
                                    password = tokens[i + 1].clone();
                                    i += 2;
                                } else {
                                    i += 1;
                                }
                            },
                            "account" | "macdef" => {
                                // Skip account and macdef (not supported)
                                i += 2;
                            },
                            _ => {
                                i += 1;
                            },
                        }
                    }

                    if !login.is_empty() && !password.is_empty() {
                        entries.insert(
                            machine.clone(),
                            NetrcEntry {
                                machine,
                                login,
                                password,
                            },
                        );
                    }
                },
                "default" => {
                    i += 1;

                    // Parse default login and password
                    let mut login = String::new();
                    let mut password = String::new();

                    while i < tokens.len() && tokens[i] != "machine" && tokens[i] != "default" {
                        match tokens[i].as_str() {
                            "login" => {
                                if i + 1 < tokens.len() {
                                    login = tokens[i + 1].clone();
                                    i += 2;
                                } else {
                                    i += 1;
                                }
                            },
                            "password" => {
                                if i + 1 < tokens.len() {
                                    password = tokens[i + 1].clone();
                                    i += 2;
                                } else {
                                    i += 1;
                                }
                            },
                            "account" | "macdef" => {
                                // Skip account and macdef (not supported)
                                i += 2;
                            },
                            _ => {
                                i += 1;
                            },
                        }
                    }

                    if !login.is_empty() && !password.is_empty() {
                        default = Some(NetrcEntry {
                            machine: "default".to_string(),
                            login,
                            password,
                        });
                    }
                },
                _ => {
                    i += 1;
                },
            }
        }

        Ok(Self { entries, default })
    }

    /// Get authentication credentials for a machine (hostname)
    ///
    /// # Arguments
    ///
    /// * `machine` - Hostname to look up (e.g., "example.com")
    ///
    /// # Returns
    ///
    /// Returns authentication configuration if found, None otherwise
    pub fn get(&self, machine: &str) -> Option<AuthConfig> {
        // Try exact match first
        if let Some(entry) = self.entries.get(machine) {
            return Some(AuthConfig {
                username: entry.login.clone(),
                password: entry.password.clone(),
                auth_type: AuthType::Basic,
            });
        }

        // Try default if no specific match
        if let Some(entry) = &self.default {
            return Some(AuthConfig {
                username: entry.login.clone(),
                password: entry.password.clone(),
                auth_type: AuthType::Basic,
            });
        }

        None
    }

    /// Check if .netrc has entry for machine
    ///
    /// # Arguments
    ///
    /// * `machine` - Hostname to check (e.g., "example.com")
    pub fn has_entry(&self, machine: &str) -> bool {
        self.entries.contains_key(machine) || self.default.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let content = r"
            machine example.com
            login myuser
            password mypass
        ";

        let netrc = Netrc::from_string(content).expect("Failed to parse netrc");
        let auth = netrc
            .get("example.com")
            .expect("Failed to get example.com auth");
        assert_eq!(auth.username, "myuser");
        assert_eq!(auth.password, "mypass");
    }

    #[test]
    fn test_parse_multiple() {
        let content = r"
            machine example.com
            login user1
            password pass1

            machine test.com
            login user2
            password pass2
        ";

        let netrc = Netrc::from_string(content).expect("Failed to parse netrc");

        let auth1 = netrc
            .get("example.com")
            .expect("Failed to get example.com auth");
        assert_eq!(auth1.username, "user1");
        assert_eq!(auth1.password, "pass1");

        let auth2 = netrc.get("test.com").expect("Failed to get test.com auth");
        assert_eq!(auth2.username, "user2");
        assert_eq!(auth2.password, "pass2");
    }

    #[test]
    fn test_parse_default() {
        let content = r"
            machine example.com
            login user1
            password pass1

            default
            login defaultuser
            password defaultpass
        ";

        let netrc = Netrc::from_string(content).expect("Failed to parse netrc");

        // Specific machine
        let auth1 = netrc
            .get("example.com")
            .expect("Failed to get example.com auth");
        assert_eq!(auth1.username, "user1");

        // Unknown machine should use default
        let auth2 = netrc
            .get("unknown.com")
            .expect("Failed to get default auth");
        assert_eq!(auth2.username, "defaultuser");
        assert_eq!(auth2.password, "defaultpass");
    }

    #[test]
    fn test_no_match() {
        let content = r"
            machine example.com
            login user1
            password pass1
        ";

        let netrc = Netrc::from_string(content).expect("Failed to parse netrc");
        assert!(netrc.get("unknown.com").is_none());
    }
}
