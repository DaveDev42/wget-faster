use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the downloader
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// Number of parallel connections for range requests
    pub parallel_chunks: usize,

    /// Size of each chunk in bytes (None for auto)
    pub chunk_size: Option<u64>,

    /// Timeout for each request
    pub timeout: Duration,

    /// Connect timeout
    pub connect_timeout: Duration,

    /// Read timeout
    pub read_timeout: Duration,

    /// User agent string
    pub user_agent: String,

    /// Retry configuration
    pub retry: RetryConfig,

    /// Proxy configuration
    pub proxy: Option<ProxyConfig>,

    /// Authentication configuration
    pub auth: Option<AuthConfig>,

    /// Custom headers
    pub headers: HashMap<String, String>,

    /// Follow redirects
    pub follow_redirects: bool,

    /// Maximum number of redirects
    pub max_redirects: usize,

    /// Enable cookies
    pub enable_cookies: bool,

    /// Cookie file path
    pub cookie_file: Option<PathBuf>,

    /// Enable compression
    pub enable_compression: bool,

    /// Verify SSL certificates
    pub verify_ssl: bool,

    /// Client certificate path
    pub client_cert: Option<PathBuf>,

    /// CA certificate path
    pub ca_cert: Option<PathBuf>,

    /// Download speed limit (bytes per second, None for unlimited)
    pub speed_limit: Option<u64>,

    /// Enable verbose logging
    pub verbose: bool,

    /// HTTP method (GET, POST, PUT, etc.)
    pub method: HttpMethod,

    /// POST/PUT data
    pub body_data: Option<Vec<u8>>,

    /// Referer URL
    pub referer: Option<String>,

    /// Content-Type for POST/PUT requests
    pub content_type: Option<String>,

    /// Enable HTTP keep-alive
    pub http_keep_alive: bool,

    /// Wait time between requests (seconds)
    pub wait_time: Option<Duration>,

    /// Random wait range multiplier (0.5-1.5x `wait_time`)
    pub random_wait: bool,

    /// Wait time between retries (seconds)
    pub wait_retry: Option<Duration>,

    /// Download quota (bytes, None for unlimited)
    pub quota: Option<u64>,

    /// Enable timestamping (only download if remote is newer)
    pub timestamping: bool,

    /// Use If-Modified-Since header
    pub if_modified_since: bool,

    /// Set local file timestamp from server
    pub use_server_timestamps: bool,

    /// Honor Content-Disposition header for filename
    pub content_disposition: bool,

    /// Save HTTP headers to output
    pub save_headers: bool,

    /// Print server response headers to stderr (wget -S style)
    pub print_server_response: bool,

    /// Send auth without waiting for challenge (preemptive auth)
    pub auth_no_challenge: bool,

    /// Save error page content even on HTTP errors (4xx/5xx)
    pub content_on_error: bool,

    /// Minimum file size threshold for parallel downloads (bytes)
    pub parallel_threshold: u64,

    /// Use pretty/modern progress output instead of wget-style (default: false for wget compatibility)
    pub pretty_output: bool,

    /// Filename restriction modes (lowercase, uppercase, nocontrol, ascii, unix, windows)
    pub restrict_file_names: Vec<FilenameRestriction>,

    /// Start downloading from this byte offset (--start-pos option)
    /// If set, overrides resume functionality from --continue
    pub start_pos: Option<u64>,

    /// Only follow HTTPS URLs (reject HTTP URLs)
    pub https_only: bool,

    /// GNU wget compatibility mode (disable HEAD requests, sequential-only)
    pub gnu_wget_compat: bool,
}

/// HTTP request method
///
/// Supported HTTP methods for download requests. Defaults to GET.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// HTTP GET - retrieve resource
    Get,
    /// HTTP HEAD - retrieve headers only
    Head,
    /// HTTP POST - submit data
    Post,
    /// HTTP PUT - update resource
    Put,
    /// HTTP DELETE - delete resource
    Delete,
    /// HTTP PATCH - partial update
    Patch,
    /// HTTP OPTIONS - query capabilities
    Options,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            parallel_chunks: 8,
            chunk_size: None, // Auto-determine
            timeout: Duration::from_secs(120),
            connect_timeout: Duration::from_secs(30),
            read_timeout: Duration::from_secs(60),
            user_agent: format!("wget-faster/{}", env!("CARGO_PKG_VERSION")),
            retry: RetryConfig::default(),
            proxy: None,
            auth: None,
            headers: HashMap::new(),
            follow_redirects: true,
            max_redirects: 20,
            enable_cookies: true,
            cookie_file: None,
            enable_compression: true,
            verify_ssl: true,
            client_cert: None,
            ca_cert: None,
            speed_limit: None,
            verbose: false,
            method: HttpMethod::Get,
            body_data: None,
            referer: None,
            content_type: None,
            http_keep_alive: true,
            wait_time: None,
            random_wait: false,
            wait_retry: None,
            quota: None,
            timestamping: false,
            if_modified_since: true,
            use_server_timestamps: true,
            content_disposition: false,
            save_headers: false,
            print_server_response: false,
            auth_no_challenge: false,
            content_on_error: false,
            parallel_threshold: 10 * 1024 * 1024, // 10MB
            pretty_output: false,                 // wget-compatible by default
            restrict_file_names: Vec::new(),      // No restrictions by default
            start_pos: None,                      // No start position by default
            https_only: false,                    // Accept both HTTP and HTTPS by default
            gnu_wget_compat: false,               // Disabled by default for performance
        }
    }
}

impl HttpMethod {
    /// Convert HTTP method to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Head => "HEAD",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Options => "OPTIONS",
        }
    }
}

impl std::str::FromStr for HttpMethod {
    type Err = String;

    /// Parse HTTP method from string (case-insensitive)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "HEAD" => Ok(HttpMethod::Head),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "PATCH" => Ok(HttpMethod::Patch),
            "OPTIONS" => Ok(HttpMethod::Options),
            _ => Err(format!("Invalid HTTP method: {s}")),
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: usize,

    /// Initial retry delay
    pub initial_delay: Duration,

    /// Maximum retry delay
    pub max_delay: Duration,

    /// Backoff multiplier
    pub backoff_multiplier: f64,

    /// Retry on connection refused
    pub retry_on_conn_refused: bool,

    /// HTTP status codes to retry on
    pub retry_on_status: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 20,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            retry_on_conn_refused: false,
            retry_on_status: vec![500, 502, 503, 504, 429],
        }
    }
}

/// Proxy configuration
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// Proxy URL
    pub url: String,

    /// Proxy authentication
    pub auth: Option<(String, String)>,

    /// Domains to bypass proxy for (`no_proxy` list)
    pub no_proxy: Vec<String>,
}

impl ProxyConfig {
    /// Check if a URL should bypass the proxy based on `no_proxy` list
    ///
    /// Implements wget's `no_proxy` matching logic:
    /// - "domain.com" matches "domain.com" and "*.domain.com"
    /// - ".domain.com" matches only "*.domain.com" (not "domain.com" itself)
    pub fn should_bypass(&self, url: &str) -> bool {
        // Parse the URL to extract the host
        let host = match url::Url::parse(url) {
            Ok(parsed) => {
                if let Some(h) = parsed.host_str() {
                    h.to_lowercase()
                } else {
                    return false;
                }
            },
            Err(_) => return false,
        };

        // Check each no_proxy pattern
        for pattern in &self.no_proxy {
            let pattern = pattern.trim().to_lowercase();
            if pattern.is_empty() {
                continue;
            }

            if let Some(domain) = pattern.strip_prefix('.') {
                // ".domain.com" matches only subdomains (e.g., "www.domain.com")
                // NOT the domain itself
                // Check if host ends with the pattern (including the dot)
                // This ensures "working2.localhost" does NOT match ".working2.localhost"
                // but "www.working2.localhost" DOES match ".working2.localhost"
                if host.ends_with(&pattern) && host != domain {
                    return true;
                }
            } else {
                // "domain.com" matches the domain AND subdomains
                if host == pattern || host.ends_with(&format!(".{pattern}")) {
                    return true;
                }
            }
        }

        false
    }
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Username
    pub username: String,

    /// Password
    pub password: String,

    /// Authentication type
    pub auth_type: AuthType,
}

/// HTTP authentication type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthType {
    /// HTTP Basic authentication (Base64-encoded credentials)
    Basic,
    /// HTTP Digest authentication (challenge-response)
    Digest,
}

/// Filename restriction modes
///
/// Controls how filenames are sanitized and transformed.
/// Multiple restrictions can be applied simultaneously.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilenameRestriction {
    /// Convert filenames to lowercase
    Lowercase,
    /// Convert filenames to uppercase
    Uppercase,
    /// Remove control characters (0x00-0x1F, 0x7F)
    NoControl,
    /// Restrict to ASCII characters (percent-encode non-ASCII)
    Ascii,
    /// Make filenames Unix-compatible (remove/replace : * ? " < > |)
    Unix,
    /// Make filenames Windows-compatible (remove/replace / \ : * ? " < > |)
    Windows,
}

impl FilenameRestriction {
    /// Apply this restriction to a filename
    pub fn apply(&self, filename: &str) -> String {
        match self {
            FilenameRestriction::Lowercase => filename.to_lowercase(),
            FilenameRestriction::Uppercase => filename.to_uppercase(),
            FilenameRestriction::NoControl => {
                filename
                    .chars()
                    .filter(|c| {
                        let byte = *c as u32;
                        // Keep characters that are NOT control characters
                        byte >= 0x20 && byte != 0x7F
                    })
                    .collect()
            },
            FilenameRestriction::Ascii => {
                filename
                    .chars()
                    .map(|c| {
                        if c.is_ascii() {
                            c.to_string()
                        } else {
                            // Percent-encode non-ASCII characters
                            format!("%{:02X}", c as u32)
                        }
                    })
                    .collect()
            },
            FilenameRestriction::Unix => {
                filename
                    .chars()
                    .map(|c| {
                        // Unix problematic characters: \ : * ? " < > |
                        match c {
                            '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                            _ => c,
                        }
                    })
                    .collect()
            },
            FilenameRestriction::Windows => {
                filename
                    .chars()
                    .map(|c| {
                        // Windows problematic characters: / \ : * ? " < > |
                        match c {
                            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                            _ => c,
                        }
                    })
                    .collect()
            },
        }
    }
}

impl std::str::FromStr for FilenameRestriction {
    type Err = String;

    /// Parse restriction mode from string (case-insensitive)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lowercase" => Ok(FilenameRestriction::Lowercase),
            "uppercase" => Ok(FilenameRestriction::Uppercase),
            "nocontrol" => Ok(FilenameRestriction::NoControl),
            "ascii" => Ok(FilenameRestriction::Ascii),
            "unix" => Ok(FilenameRestriction::Unix),
            "windows" => Ok(FilenameRestriction::Windows),
            _ => Err(format!("Invalid filename restriction: {s}")),
        }
    }
}

/// Apply a list of filename restrictions in order
pub fn apply_filename_restrictions(filename: &str, restrictions: &[FilenameRestriction]) -> String {
    restrictions
        .iter()
        .fold(filename.to_string(), |name, restriction| restriction.apply(&name))
}
