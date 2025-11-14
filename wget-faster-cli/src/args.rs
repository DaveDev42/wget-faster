use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "wgetf")]
#[command(
    version,
    about = "GNU Wget compatible downloader with high-performance parallel downloads",
    long_about = None,
    disable_version_flag = true
)]
pub struct Args {
    /// URLs to download
    #[arg(value_name = "URL")]
    pub urls: Vec<String>,

    // ===== Startup Options =====
    /// Display version information
    #[arg(short = 'V', long = "version", overrides_with = "version")]
    pub version: bool,

    /// Display help
    #[arg(short = 'h', long, overrides_with = "help")]
    pub help: bool,

    /// Go to background after startup
    #[arg(short = 'b', long, overrides_with = "background")]
    pub background: bool,

    /// Execute a `.wgetrc'-style command
    #[arg(short = 'e', long, value_name = "COMMAND")]
    pub execute: Option<String>,

    // ===== Logging and Input File Options =====
    /// Log messages to FILE
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output_file: Option<PathBuf>,

    /// Append messages to FILE
    #[arg(short = 'a', long, value_name = "FILE")]
    pub append_output: Option<PathBuf>,

    /// Print lots of debugging information
    #[arg(short = 'd', long, action = clap::ArgAction::Count, overrides_with = "debug")]
    pub debug: u8,

    /// Quiet (no output)
    #[arg(short = 'q', long, overrides_with = "quiet")]
    pub quiet: bool,

    /// Be verbose (this is the default)
    #[arg(short = 'v', long, overrides_with = "verbose")]
    pub verbose: bool,

    /// Turn off verboseness, without being quiet
    #[arg(long, overrides_with = "no_verbose")]
    pub no_verbose: bool,

    /// Output bandwidth as TYPE
    #[arg(long, value_name = "TYPE")]
    pub report_speed: Option<String>,

    /// Download URLs found in local or external FILE
    #[arg(short = 'i', long, value_name = "FILE")]
    pub input_file: Option<PathBuf>,

    /// Treat input file as HTML
    #[arg(short = 'F', long, overrides_with = "force_html")]
    pub force_html: bool,

    /// Resolves HTML input-file links relative to URL
    #[arg(short = 'B', long, value_name = "URL")]
    pub base: Option<String>,

    /// Specify config file to use
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Do not read any config file
    #[arg(long, overrides_with = "no_config")]
    pub no_config: bool,

    /// Log reasons for URL rejection to FILE
    #[arg(long, value_name = "FILE")]
    pub rejected_log: Option<PathBuf>,

    // ===== Download Options =====
    /// Set number of retries to NUMBER (0 unlimits)
    #[arg(short = 't', long, value_name = "NUMBER", default_value = "20")]
    pub tries: usize,

    /// Retry even if connection is refused
    #[arg(long, overrides_with = "retry_connrefused")]
    pub retry_connrefused: bool,

    /// Consider host errors as non-fatal, transient errors
    #[arg(long, overrides_with = "retry_on_host_error")]
    pub retry_on_host_error: bool,

    /// Comma-separated list of HTTP errors to retry
    #[arg(long, value_name = "ERRORS")]
    pub retry_on_http_error: Option<String>,

    /// Write documents to FILE
    #[arg(short = 'O', long, value_name = "FILE")]
    pub output_document: Option<PathBuf>,

    /// Skip downloads that would download to existing files
    #[arg(long, overrides_with = "no_clobber")]
    pub no_clobber: bool,

    /// Don't try to obtain credentials from .netrc
    #[arg(long, overrides_with = "no_netrc")]
    pub no_netrc: bool,

    /// Resume getting a partially-downloaded file
    #[arg(short = 'c', long, alias = "continue", overrides_with = "continue_download")]
    pub continue_download: bool,

    /// Start downloading from zero-based position OFFSET
    #[arg(long, value_name = "OFFSET")]
    pub start_pos: Option<u64>,

    /// Select progress gauge type
    #[arg(long, value_name = "TYPE")]
    pub progress: Option<String>,

    /// Display the progress bar in any verbosity mode
    #[arg(long, overrides_with = "show_progress")]
    pub show_progress: bool,

    /// Don't re-retrieve files unless newer than local
    #[arg(short = 'N', long, overrides_with = "timestamping")]
    pub timestamping: bool,

    /// Don't use conditional if-modified-since get requests
    #[arg(long, overrides_with = "no_if_modified_since")]
    pub no_if_modified_since: bool,

    /// Don't set the local file's timestamp by the one on the server
    #[arg(long, overrides_with = "no_use_server_timestamps")]
    pub no_use_server_timestamps: bool,

    /// Print server response
    #[arg(short = 'S', long, overrides_with = "server_response")]
    pub server_response: bool,

    /// Don't download anything
    #[arg(long, overrides_with = "spider")]
    pub spider: bool,

    /// Set all timeout values to SECONDS
    #[arg(short = 'T', long, value_name = "SECONDS")]
    pub timeout: Option<u64>,

    /// Set the DNS lookup timeout to SECS
    #[arg(long, value_name = "SECS")]
    pub dns_timeout: Option<u64>,

    /// Set the connect timeout to SECS
    #[arg(long, value_name = "SECS")]
    pub connect_timeout: Option<u64>,

    /// Set the read timeout to SECS
    #[arg(long, value_name = "SECS")]
    pub read_timeout: Option<u64>,

    /// Wait SECONDS between retrievals
    #[arg(short = 'w', long, value_name = "SECONDS")]
    pub wait: Option<u64>,

    /// Wait 1..SECONDS between retries of a retrieval
    #[arg(long, value_name = "SECONDS")]
    pub waitretry: Option<u64>,

    /// Wait from 0.5*WAIT...1.5*WAIT secs between retrievals
    #[arg(long, overrides_with = "random_wait")]
    pub random_wait: bool,

    /// Explicitly turn off proxy
    #[arg(long, overrides_with = "no_proxy")]
    pub no_proxy: bool,

    /// Set retrieval quota to NUMBER
    #[arg(short = 'Q', long, value_name = "NUMBER")]
    pub quota: Option<String>,

    /// Bind to ADDRESS (hostname or IP) on local host
    #[arg(long, value_name = "ADDRESS")]
    pub bind_address: Option<String>,

    /// Limit download rate to RATE
    #[arg(long, value_name = "RATE")]
    pub limit_rate: Option<String>,

    /// Disable caching DNS lookups
    #[arg(long, overrides_with = "no_dns_cache")]
    pub no_dns_cache: bool,

    /// Restrict chars in file names to ones OS allows
    #[arg(long, value_name = "OS")]
    pub restrict_file_names: Option<String>,

    /// Ignore case when matching files/directories
    #[arg(long, overrides_with = "ignore_case")]
    pub ignore_case: bool,

    /// Connect only to IPv4 addresses
    #[arg(short = '4', long, overrides_with = "inet4_only")]
    pub inet4_only: bool,

    /// Connect only to IPv6 addresses
    #[arg(short = '6', long, overrides_with = "inet6_only")]
    pub inet6_only: bool,

    /// Connect first to addresses of specified family
    #[arg(long, value_name = "FAMILY")]
    pub prefer_family: Option<String>,

    /// Set both ftp and http user to USER
    #[arg(long, value_name = "USER")]
    pub user: Option<String>,

    /// Set both ftp and http password to PASS
    #[arg(long, value_name = "PASS")]
    pub password: Option<String>,

    /// Prompt for passwords
    #[arg(long, overrides_with = "ask_password")]
    pub ask_password: bool,

    /// Specify credential handler for requesting username and password
    #[arg(long, value_name = "COMMAND")]
    pub use_askpass: Option<String>,

    /// Turn off IRI support
    #[arg(long, overrides_with = "no_iri")]
    pub no_iri: bool,

    /// Use ENC as the local encoding for IRIs
    #[arg(long, value_name = "ENC")]
    pub local_encoding: Option<String>,

    /// Use ENC as the default remote encoding
    #[arg(long, value_name = "ENC")]
    pub remote_encoding: Option<String>,

    /// Remove file before clobber
    #[arg(long, overrides_with = "unlink")]
    pub unlink: bool,

    /// Turn on storage of metadata in extended file attributes
    #[arg(long, overrides_with = "xattr")]
    pub xattr: bool,

    // ===== Directory Options =====
    /// Don't create directories
    #[arg(short = 'n', long, action = clap::ArgAction::SetTrue, overrides_with = "no_directories")]
    pub no_directories: bool,

    /// Force creation of directories
    #[arg(short = 'x', long, overrides_with = "force_directories")]
    pub force_directories: bool,

    /// Don't create host directories
    #[arg(long, action = clap::ArgAction::SetTrue, overrides_with = "no_host_directories")]
    pub no_host_directories: bool,

    /// Use protocol name in directories
    #[arg(long, overrides_with = "protocol_directories")]
    pub protocol_directories: bool,

    /// Save files to PREFIX/..
    #[arg(short = 'P', long, value_name = "PREFIX")]
    pub directory_prefix: Option<PathBuf>,

    /// Ignore NUMBER remote directory components
    #[arg(long, value_name = "NUMBER")]
    pub cut_dirs: Option<usize>,

    // ===== HTTP Options =====
    /// Set http user to USER
    #[arg(long, value_name = "USER")]
    pub http_user: Option<String>,

    /// Set http password to PASS
    #[arg(long, value_name = "PASS")]
    pub http_password: Option<String>,

    /// Disallow server-cached data
    #[arg(long, overrides_with = "no_cache")]
    pub no_cache: bool,

    /// Change the default page name
    #[arg(long, value_name = "NAME")]
    pub default_page: Option<String>,

    /// Save HTML/CSS documents with proper extensions
    #[arg(short = 'E', long, overrides_with = "adjust_extension")]
    pub adjust_extension: bool,

    /// Ignore 'Content-Length' header field
    #[arg(long, overrides_with = "ignore_length")]
    pub ignore_length: bool,

    /// Insert STRING among the headers
    #[arg(long, value_name = "STRING")]
    pub header: Vec<String>,

    /// Choose compression type
    #[arg(long, value_name = "TYPE")]
    pub compression: Option<String>,

    /// Maximum redirections allowed per page
    #[arg(long, value_name = "NUM")]
    pub max_redirect: Option<usize>,

    /// Set USER as proxy username
    #[arg(long, value_name = "USER")]
    pub proxy_user: Option<String>,

    /// Set PASS as proxy password
    #[arg(long, value_name = "PASS")]
    pub proxy_password: Option<String>,

    /// Include 'Referer: URL' header in HTTP request
    #[arg(long, value_name = "URL")]
    pub referer: Option<String>,

    /// Save the HTTP headers to file
    #[arg(long, overrides_with = "save_headers")]
    pub save_headers: bool,

    /// Identify as AGENT instead of Wget/VERSION
    #[arg(short = 'U', long, value_name = "AGENT")]
    pub user_agent: Option<String>,

    /// Disable HTTP keep-alive
    #[arg(long, overrides_with = "no_http_keep_alive")]
    pub no_http_keep_alive: bool,

    /// Don't use cookies
    #[arg(long, overrides_with = "no_cookies")]
    pub no_cookies: bool,

    /// Load cookies from FILE before session
    #[arg(long, value_name = "FILE")]
    pub load_cookies: Option<PathBuf>,

    /// Save cookies to FILE after session
    #[arg(long, value_name = "FILE")]
    pub save_cookies: Option<PathBuf>,

    /// Load and save session (non-permanent) cookies
    #[arg(long, overrides_with = "keep_session_cookies")]
    pub keep_session_cookies: bool,

    /// Use the POST method; send STRING as the data
    #[arg(long, value_name = "STRING")]
    pub post_data: Option<String>,

    /// Use the POST method; send contents of FILE
    #[arg(long, value_name = "FILE")]
    pub post_file: Option<PathBuf>,

    /// Use method "HTTPMethod" in the request
    #[arg(long, value_name = "HTTPMethod")]
    pub method: Option<String>,

    /// Send STRING as data
    #[arg(long, value_name = "STRING")]
    pub body_data: Option<String>,

    /// Send contents of FILE
    #[arg(long, value_name = "FILE")]
    pub body_file: Option<PathBuf>,

    /// Honor the Content-Disposition header
    #[arg(long, overrides_with = "content_disposition")]
    pub content_disposition: bool,

    /// Ignore the Content-Disposition header
    #[arg(long, overrides_with = "no_content_disposition")]
    pub no_content_disposition: bool,

    /// Output the received content on server errors
    #[arg(long, overrides_with = "content_on_error")]
    pub content_on_error: bool,

    /// Send Basic HTTP authentication without waiting for challenge
    #[arg(long, overrides_with = "auth_no_challenge")]
    pub auth_no_challenge: bool,

    // ===== HTTPS (SSL/TLS) Options =====
    /// Choose secure protocol
    #[arg(long, value_name = "PR")]
    pub secure_protocol: Option<String>,

    /// Only follow secure HTTPS links
    #[arg(long, overrides_with = "https_only")]
    pub https_only: bool,

    /// Don't validate the server's certificate
    #[arg(long, overrides_with = "no_check_certificate")]
    pub no_check_certificate: bool,

    /// Client certificate file
    #[arg(long, value_name = "FILE")]
    pub certificate: Option<PathBuf>,

    /// Client certificate type, PEM or DER
    #[arg(long, value_name = "TYPE")]
    pub certificate_type: Option<String>,

    /// Private key file
    #[arg(long, value_name = "FILE")]
    pub private_key: Option<PathBuf>,

    /// Private key type, PEM or DER
    #[arg(long, value_name = "TYPE")]
    pub private_key_type: Option<String>,

    /// File with the bundle of CAs
    #[arg(long, value_name = "FILE")]
    pub ca_certificate: Option<PathBuf>,

    /// Directory where hash list of CAs is stored
    #[arg(long, value_name = "DIR")]
    pub ca_directory: Option<PathBuf>,

    /// File with bundle of CRLs
    #[arg(long, value_name = "FILE")]
    pub crl_file: Option<PathBuf>,

    /// Public key file to verify peer against
    #[arg(long, value_name = "FILE/HASHES")]
    pub pinnedpubkey: Option<String>,

    /// File with random data for seeding the SSL PRNG
    #[arg(long, value_name = "FILE")]
    pub random_file: Option<PathBuf>,

    /// Set the priority string or cipher list string directly
    #[arg(long, value_name = "STR")]
    pub ciphers: Option<String>,

    // ===== HSTS Options =====
    /// Disable HSTS
    #[arg(long, overrides_with = "no_hsts")]
    pub no_hsts: bool,

    /// Path of HSTS database
    #[arg(long, value_name = "FILE")]
    pub hsts_file: Option<PathBuf>,

    // ===== FTP Options =====
    /// Set ftp user to USER
    #[arg(long, value_name = "USER")]
    pub ftp_user: Option<String>,

    /// Set ftp password to PASS
    #[arg(long, value_name = "PASS")]
    pub ftp_password: Option<String>,

    /// Don't remove '.listing' files
    #[arg(long, overrides_with = "no_remove_listing")]
    pub no_remove_listing: bool,

    /// Turn off FTP file name globbing
    #[arg(long, overrides_with = "no_glob")]
    pub no_glob: bool,

    /// Disable the "passive" transfer mode
    #[arg(long, overrides_with = "no_passive_ftp")]
    pub no_passive_ftp: bool,

    /// Preserve remote file permissions
    #[arg(long, overrides_with = "preserve_permissions")]
    pub preserve_permissions: bool,

    /// When recursing, get linked-to files (not dir)
    #[arg(long, overrides_with = "retr_symlinks")]
    pub retr_symlinks: bool,

    // ===== FTPS Options =====
    /// Use implicit FTPS (default port is 990)
    #[arg(long, overrides_with = "ftps_implicit")]
    pub ftps_implicit: bool,

    /// Resume the SSL/TLS session started in the control connection
    #[arg(long, overrides_with = "ftps_resume_ssl")]
    pub ftps_resume_ssl: bool,

    /// Cipher the control channel only
    #[arg(long, overrides_with = "ftps_clear_data_connection")]
    pub ftps_clear_data_connection: bool,

    /// Fall back to FTP if FTPS is not supported
    #[arg(long, overrides_with = "ftps_fallback_to_ftp")]
    pub ftps_fallback_to_ftp: bool,

    // ===== Recursive Download Options =====
    /// Specify recursive download
    #[arg(short = 'r', long, overrides_with = "recursive")]
    pub recursive: bool,

    /// Maximum recursion depth (inf or 0 for infinite)
    #[arg(short = 'l', long, value_name = "NUMBER")]
    pub level: Option<String>,

    /// Delete files locally after downloading them
    #[arg(long, overrides_with = "delete_after")]
    pub delete_after: bool,

    /// Make links in downloaded HTML or CSS point to local files
    #[arg(short = 'k', long, overrides_with = "convert_links")]
    pub convert_links: bool,

    /// Convert the file part of the URLs only
    #[arg(long, overrides_with = "convert_file_only")]
    pub convert_file_only: bool,

    /// Before writing file X, rotate up to N backup files
    #[arg(long, value_name = "N")]
    pub backups: Option<usize>,

    /// Before converting file X, back up as X.orig
    #[arg(short = 'K', long, overrides_with = "backup_converted")]
    pub backup_converted: bool,

    /// Shortcut for -N -r -l inf --no-remove-listing
    #[arg(short = 'm', long, overrides_with = "mirror")]
    pub mirror: bool,

    /// Get all images, etc. needed to display HTML page
    #[arg(short = 'p', long, overrides_with = "page_requisites")]
    pub page_requisites: bool,

    /// Turn on strict (SGML) handling of HTML comments
    #[arg(long, overrides_with = "strict_comments")]
    pub strict_comments: bool,

    // ===== Recursive Accept/Reject Options =====
    /// Comma-separated list of accepted extensions
    #[arg(short = 'A', long, value_name = "LIST")]
    pub accept: Option<String>,

    /// Comma-separated list of rejected extensions
    #[arg(short = 'R', long, value_name = "LIST")]
    pub reject: Option<String>,

    /// Regex matching accepted URLs
    #[arg(long, value_name = "REGEX")]
    pub accept_regex: Option<String>,

    /// Regex matching rejected URLs
    #[arg(long, value_name = "REGEX")]
    pub reject_regex: Option<String>,

    /// Regex type (posix)
    #[arg(long, value_name = "TYPE")]
    pub regex_type: Option<String>,

    /// Comma-separated list of accepted domains
    #[arg(short = 'D', long, value_name = "LIST")]
    pub domains: Option<String>,

    /// Comma-separated list of rejected domains
    #[arg(long, value_name = "LIST")]
    pub exclude_domains: Option<String>,

    /// Follow FTP links from HTML documents
    #[arg(long, overrides_with = "follow_ftp")]
    pub follow_ftp: bool,

    /// Comma-separated list of followed HTML tags
    #[arg(long, value_name = "LIST")]
    pub follow_tags: Option<String>,

    /// Comma-separated list of ignored HTML tags
    #[arg(long, value_name = "LIST")]
    pub ignore_tags: Option<String>,

    /// Go to foreign hosts when recursive
    #[arg(short = 'H', long, overrides_with = "span_hosts")]
    pub span_hosts: bool,

    /// Follow relative links only
    #[arg(short = 'L', long, overrides_with = "relative")]
    pub relative: bool,

    /// List of allowed directories
    #[arg(short = 'I', long, value_name = "LIST")]
    pub include_directories: Option<String>,

    /// Use the name specified by the redirection URL's last component
    #[arg(long, overrides_with = "trust_server_names")]
    pub trust_server_names: bool,

    /// List of excluded directories
    #[arg(short = 'X', long, value_name = "LIST")]
    pub exclude_directories: Option<String>,

    /// Don't ascend to the parent directory
    #[arg(long, overrides_with = "no_parent")]
    pub no_parent: bool,
}

impl Args {
    /// Validate argument combinations
    pub fn validate(&self) -> Result<(), String> {
        // -N and -O are incompatible
        if self.timestamping && self.output_document.is_some() {
            return Err("-N (timestamping) cannot be used with -O (output-document)".to_string());
        }

        // -k only makes sense with -O when downloading a single document
        if self.convert_links && self.output_document.is_some() && self.urls.len() > 1 {
            return Err("-k (convert-links) with -O (output-document) only works with single URL".to_string());
        }

        // Spider mode doesn't need output
        if self.spider && self.output_document.is_some() {
            return Err("--spider mode doesn't download, -O makes no sense".to_string());
        }

        Ok(())
    }
}
