use crate::{Error, Result};
use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// A cookie with all its attributes
#[derive(Debug, Clone)]
pub struct Cookie {
    pub domain: String,
    pub include_subdomains: bool,
    pub path: String,
    pub secure: bool,
    pub expiration: Option<u64>, // Unix timestamp, None for session cookies
    pub name: String,
    pub value: String,
}

/// Cookie jar that stores cookies
#[derive(Debug, Clone, Default)]
pub struct CookieJar {
    cookies: HashMap<String, Vec<Cookie>>,
}

impl CookieJar {
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }

    /// Add a cookie to the jar
    pub fn add_cookie(&mut self, cookie: Cookie) {
        let domain_key = cookie.domain.to_lowercase();
        self.cookies
            .entry(domain_key)
            .or_insert_with(Vec::new)
            .push(cookie);
    }

    /// Get cookies for a domain
    pub fn get_cookies_for_domain(&self, domain: &str) -> Vec<&Cookie> {
        let domain_lower = domain.to_lowercase();
        let mut result = Vec::new();

        for (jar_domain, cookies) in &self.cookies {
            if domain_matches(&domain_lower, jar_domain) {
                for cookie in cookies {
                    // Check if cookie is expired
                    if let Some(expiration) = cookie.expiration {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        if now > expiration {
                            continue; // Skip expired cookies
                        }
                    }
                    result.push(cookie);
                }
            }
        }

        result
    }

    /// Load cookies from a Netscape format file
    pub async fn load_from_file(path: &Path) -> Result<Self> {
        let file = File::open(path).await?;
        let reader = BufReader::new(file);
        let mut jar = CookieJar::new();

        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Netscape format: domain flag path secure expiration name value
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() != 7 {
                continue; // Skip malformed lines
            }

            let domain = parts[0].to_string();
            let include_subdomains = parts[1] == "TRUE";
            let path = parts[2].to_string();
            let secure = parts[3] == "TRUE";
            let expiration = parts[4].parse::<u64>().ok();
            let name = parts[5].to_string();
            let value = parts[6].to_string();

            jar.add_cookie(Cookie {
                domain,
                include_subdomains,
                path,
                secure,
                expiration,
                name,
                value,
            });
        }

        Ok(jar)
    }

    /// Save cookies to a Netscape format file
    pub async fn save_to_file(&self, path: &Path) -> Result<()> {
        let mut file = File::create(path).await?;

        // Write header
        file.write_all(b"# Netscape HTTP Cookie File\n").await?;
        file.write_all(b"# This is a generated file! Do not edit.\n\n")
            .await?;

        // Write cookies
        for cookies in self.cookies.values() {
            for cookie in cookies {
                let line = format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    cookie.domain,
                    if cookie.include_subdomains {
                        "TRUE"
                    } else {
                        "FALSE"
                    },
                    cookie.path,
                    if cookie.secure { "TRUE" } else { "FALSE" },
                    cookie.expiration.unwrap_or(0),
                    cookie.name,
                    cookie.value
                );
                file.write_all(line.as_bytes()).await?;
            }
        }

        file.flush().await?;
        Ok(())
    }

    /// Get all cookies as a Cookie header value
    pub fn to_cookie_header(&self, domain: &str, path: &str, secure: bool) -> Option<String> {
        let cookies = self.get_cookies_for_domain(domain);

        let mut matching_cookies = Vec::new();

        for cookie in cookies {
            // Check path matching
            if !path.starts_with(&cookie.path) {
                continue;
            }

            // Check secure flag
            if cookie.secure && !secure {
                continue;
            }

            matching_cookies.push(format!("{}={}", cookie.name, cookie.value));
        }

        if matching_cookies.is_empty() {
            None
        } else {
            Some(matching_cookies.join("; "))
        }
    }

    /// Parse and add cookies from Set-Cookie headers
    pub fn add_from_set_cookie(&mut self, domain: &str, set_cookie: &str) {
        // Parse Set-Cookie header
        // Format: name=value; Domain=...; Path=...; Secure; HttpOnly; Expires=...; Max-Age=...

        let parts: Vec<&str> = set_cookie.split(';').collect();
        if parts.is_empty() {
            return;
        }

        // Parse name=value
        let name_value: Vec<&str> = parts[0].split('=').collect();
        if name_value.len() != 2 {
            return;
        }

        let name = name_value[0].trim().to_string();
        let value = name_value[1].trim().to_string();

        let mut cookie = Cookie {
            domain: domain.to_string(),
            include_subdomains: true,
            path: "/".to_string(),
            secure: false,
            expiration: None,
            name,
            value,
        };

        // Parse attributes
        for part in &parts[1..] {
            let part = part.trim();

            if part.to_lowercase() == "secure" {
                cookie.secure = true;
            } else if part.to_lowercase().starts_with("path=") {
                cookie.path = part[5..].trim().to_string();
            } else if part.to_lowercase().starts_with("domain=") {
                cookie.domain = part[7..].trim().to_string();
            } else if part.to_lowercase().starts_with("expires=") {
                // Parse Expires date (we'll skip this for simplicity)
            } else if part.to_lowercase().starts_with("max-age=") {
                if let Ok(max_age) = part[8..].trim().parse::<u64>() {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    cookie.expiration = Some(now + max_age);
                }
            }
        }

        self.add_cookie(cookie);
    }
}

/// Check if a domain matches a cookie domain
fn domain_matches(request_domain: &str, cookie_domain: &str) -> bool {
    if request_domain == cookie_domain {
        return true;
    }

    // Check if request_domain is a subdomain of cookie_domain
    if cookie_domain.starts_with('.') {
        request_domain.ends_with(cookie_domain) || request_domain == &cookie_domain[1..]
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_matching() {
        assert!(domain_matches("example.com", "example.com"));
        assert!(domain_matches("www.example.com", ".example.com"));
        assert!(domain_matches("example.com", ".example.com"));
        assert!(!domain_matches("example.com", "other.com"));
        assert!(!domain_matches("example.com", ".other.com"));
    }

    #[test]
    fn test_cookie_jar() {
        let mut jar = CookieJar::new();

        jar.add_cookie(Cookie {
            domain: "example.com".to_string(),
            include_subdomains: true,
            path: "/".to_string(),
            secure: false,
            expiration: None,
            name: "session".to_string(),
            value: "abc123".to_string(),
        });

        let cookies = jar.get_cookies_for_domain("example.com");
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "session");
        assert_eq!(cookies[0].value, "abc123");
    }
}
