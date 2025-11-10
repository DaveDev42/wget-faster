use std::time::Duration;
use std::path::PathBuf;
use std::collections::HashMap;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthType {
    Basic,
    Digest,
}
