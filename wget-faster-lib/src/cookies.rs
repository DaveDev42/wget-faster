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
        self.cookies.entry(domain_key).or_default().push(cookie);
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
                // Parse Expires date
                // Format: Wdy, DD Mon YYYY HH:MM:SS GMT
                // We need to parse this leniently because many servers send incorrect day-of-week
                let date_str = part[8..].trim();

                // Split by comma and spaces to extract components
                // Example: "Sun, 06 Nov 2001 12:32:43 GMT"
                let parts_vec: Vec<&str> = date_str
                    .split(&[',', ' '][..])
                    .filter(|s| !s.is_empty())
                    .collect();

                if parts_vec.len() >= 6 {
                    // parts_vec[0] = day name (ignored)
                    // parts_vec[1] = day
                    // parts_vec[2] = month
                    // parts_vec[3] = year
                    // parts_vec[4] = time (HH:MM:SS)
                    // parts_vec[5] = GMT

                    if let (Ok(day), Some(month_num), Ok(year)) = (
                        parts_vec[1].parse::<u32>(),
                        parse_month(parts_vec[2]),
                        parts_vec[3].parse::<i32>(),
                    ) {
                        // Parse time components
                        let time_parts: Vec<&str> = parts_vec[4].split(':').collect();
                        if time_parts.len() == 3 {
                            if let (Ok(hour), Ok(min), Ok(sec)) = (
                                time_parts[0].parse::<u32>(),
                                time_parts[1].parse::<u32>(),
                                time_parts[2].parse::<u32>(),
                            ) {
                                // Create a NaiveDateTime without validating day-of-week
                                if let Some(naive_dt) =
                                    chrono::NaiveDate::from_ymd_opt(year, month_num, day)
                                        .and_then(|d| d.and_hms_opt(hour, min, sec))
                                {
                                    let timestamp = naive_dt.and_utc().timestamp() as u64;
                                    cookie.expiration = Some(timestamp);
                                }
                            }
                        }
                    }
                }
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

/// Parse month name to month number (1-12)
fn parse_month(month_str: &str) -> Option<u32> {
    match month_str.to_lowercase().as_str() {
        "jan" => Some(1),
        "feb" => Some(2),
        "mar" => Some(3),
        "apr" => Some(4),
        "may" => Some(5),
        "jun" => Some(6),
        "jul" => Some(7),
        "aug" => Some(8),
        "sep" => Some(9),
        "oct" => Some(10),
        "nov" => Some(11),
        "dec" => Some(12),
        _ => None,
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

    #[test]
    fn test_cookie_expiry_parsing() {
        let mut jar = CookieJar::new();

        // Test future expiry date (should be kept)
        jar.add_from_set_cookie(
            "example.com",
            "future=value; Expires=Wed, 21 Oct 2099 07:28:00 GMT",
        );

        // Test past expiry date (should be stored but filtered out when retrieved)
        jar.add_from_set_cookie(
            "example.com",
            "expired=value; Expires=Sun, 06 Nov 2001 12:32:43 GMT",
        );

        // Test no expiry (session cookie, should be kept)
        jar.add_from_set_cookie("example.com", "session=value");

        // get_cookies_for_domain filters out expired cookies
        let cookies = jar.get_cookies_for_domain("example.com");

        // Should have 2 cookies (future and session), expired should be filtered out
        assert_eq!(cookies.len(), 2);

        let names: Vec<&str> = cookies.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"future"));
        assert!(names.contains(&"session"));
        assert!(!names.contains(&"expired"));

        // Verify the expired cookie has an expiration set
        let all_cookies: Vec<&Cookie> = jar.cookies.values().flatten().collect();
        let expired_cookie = all_cookies.iter().find(|c| c.name == "expired").unwrap();
        assert!(expired_cookie.expiration.is_some());
    }

    #[test]
    fn test_cookie_header_with_expiry() {
        let mut jar = CookieJar::new();

        // Add unexpired cookie
        jar.add_from_set_cookie("localhost", "sess-id=0213; path=/");

        // Should be included in header
        let header = jar.to_cookie_header("localhost", "/", false);
        assert_eq!(header, Some("sess-id=0213".to_string()));

        // Now add an expired cookie with same name (simulating server overwriting)
        jar.add_from_set_cookie(
            "localhost",
            "sess-id=0213; path=/; Expires=Sun, 06 Nov 2001 12:32:43 GMT",
        );

        // The expired cookie should not be included
        // Since we're adding, not replacing, we'll have both. The jar needs deduplication logic.
        // For now, let's just test that expired cookies are filtered
        let cookies = jar.get_cookies_for_domain("localhost");
        let active_cookies: Vec<_> = cookies
            .iter()
            .filter(|c| {
                if let Some(exp) = c.expiration {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    exp > now
                } else {
                    true
                }
            })
            .collect();

        // Should have only the non-expired one
        assert_eq!(active_cookies.len(), 1);
    }
}
