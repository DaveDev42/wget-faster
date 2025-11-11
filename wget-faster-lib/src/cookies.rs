use crate::Result;
use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// An HTTP cookie with all its attributes
///
/// Represents a single HTTP cookie with domain, path, security settings,
/// and optional expiration time.
///
/// # Examples
///
/// ```
/// use wget_faster_lib::cookies::Cookie;
///
/// let cookie = Cookie {
///     domain: "example.com".to_string(),
///     include_subdomains: true,
///     path: "/".to_string(),
///     secure: false,
///     expiration: None,  // Session cookie
///     name: "session_id".to_string(),
///     value: "abc123".to_string(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Cookie {
    /// Domain this cookie applies to (e.g., "example.com")
    pub domain: String,

    /// Whether this cookie should be sent to subdomains
    pub include_subdomains: bool,

    /// Path this cookie applies to (e.g., "/", "/api")
    pub path: String,

    /// Whether this cookie should only be sent over HTTPS
    pub secure: bool,

    /// Expiration time as Unix timestamp, None for session cookies
    pub expiration: Option<u64>,

    /// Cookie name
    pub name: String,

    /// Cookie value
    pub value: String,
}

/// Cookie jar for managing HTTP cookies
///
/// Stores and manages cookies according to domain and path rules.
/// Supports loading/saving cookies in Netscape format (compatible with wget/curl).
///
/// # Examples
///
/// ```no_run
/// use wget_faster_lib::cookies::{CookieJar, Cookie};
/// use std::path::Path;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Load cookies from file
///     let jar = CookieJar::load_from_file(Path::new("cookies.txt")).await?;
///
///     // Get Cookie header for a domain
///     if let Some(header) = jar.to_cookie_header("example.com", "/", false) {
///         println!("Cookie: {}", header);
///     }
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct CookieJar {
    cookies: HashMap<String, Vec<Cookie>>,
}

impl CookieJar {
    /// Create a new empty cookie jar
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

    /// Load cookies from a Netscape format cookie file
    ///
    /// Reads cookies from a file in Netscape cookie format (compatible with wget/curl).
    /// Automatically skips expired cookies and malformed lines.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the cookie file
    ///
    /// # Returns
    ///
    /// A new `CookieJar` containing the loaded cookies
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read
    ///
    /// # Netscape Cookie Format
    ///
    /// Each line contains tab-separated fields:
    /// `domain` `flag` `path` `secure` `expiration` `name` `value`
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
    ///
    /// Writes all cookies to a file in Netscape cookie format (compatible with wget/curl).
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the cookie file will be saved
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written
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

    /// Convert cookies to a Cookie header value
    ///
    /// Builds a Cookie header value for a specific domain, path, and security context.
    /// Only includes cookies that match the domain/path and aren't expired.
    ///
    /// # Arguments
    ///
    /// * `domain` - The domain of the request
    /// * `path` - The path of the request
    /// * `secure` - Whether the request uses HTTPS
    ///
    /// # Returns
    ///
    /// `Some(String)` with the Cookie header value, or `None` if no cookies match
    ///
    /// # Examples
    ///
    /// ```
    /// use wget_faster_lib::cookies::{CookieJar, Cookie};
    ///
    /// let mut jar = CookieJar::new();
    /// jar.add_cookie(Cookie {
    ///     domain: "example.com".to_string(),
    ///     include_subdomains: true,
    ///     path: "/".to_string(),
    ///     secure: false,
    ///     expiration: None,
    ///     name: "session".to_string(),
    ///     value: "abc123".to_string(),
    /// });
    ///
    /// let header = jar.to_cookie_header("example.com", "/api", false);
    /// assert_eq!(header, Some("session=abc123".to_string()));
    /// ```
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

    /// Parse and add cookies from Set-Cookie header
    ///
    /// Parses a Set-Cookie header value and adds the cookie to the jar.
    /// Supports standard cookie attributes like Domain, Path, Secure, Max-Age.
    ///
    /// # Arguments
    ///
    /// * `domain` - Default domain if not specified in the cookie
    /// * `set_cookie` - The Set-Cookie header value
    ///
    /// # Examples
    ///
    /// ```
    /// use wget_faster_lib::cookies::CookieJar;
    ///
    /// let mut jar = CookieJar::new();
    /// jar.add_from_set_cookie(
    ///     "example.com",
    ///     "session=xyz789; Path=/; Secure; Max-Age=3600"
    /// );
    ///
    /// let header = jar.to_cookie_header("example.com", "/", true);
    /// assert!(header.is_some());
    /// ```
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
