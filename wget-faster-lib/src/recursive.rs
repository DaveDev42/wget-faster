/// Recursive download functionality for downloading entire websites

use crate::{Error, Result, Downloader, DownloadConfig};
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use url::Url;

/// Configuration for recursive downloads
#[derive(Debug, Clone)]
pub struct RecursiveConfig {
    /// Maximum recursion depth (0 = infinite)
    pub max_depth: usize,

    /// Follow links across domains
    pub span_hosts: bool,

    /// Only follow relative links
    pub relative_only: bool,

    /// Convert links for local viewing
    pub convert_links: bool,

    /// Download page requisites (images, CSS, JS)
    pub page_requisites: bool,

    /// Accepted file extensions (empty = all)
    pub accept_extensions: Vec<String>,

    /// Rejected file extensions
    pub reject_extensions: Vec<String>,

    /// Accepted domains
    pub accepted_domains: Vec<String>,

    /// Rejected domains
    pub rejected_domains: Vec<String>,

    /// Include directories (path patterns)
    pub include_directories: Vec<String>,

    /// Exclude directories (path patterns)
    pub exclude_directories: Vec<String>,

    /// Don't ascend to parent directory
    pub no_parent: bool,

    /// Don't create hostname directories
    pub no_host_directories: bool,

    /// Spider mode - only check links, don't download files
    pub spider: bool,
}

impl Default for RecursiveConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            span_hosts: false,
            relative_only: false,
            convert_links: false,
            page_requisites: false,
            accept_extensions: Vec::new(),
            reject_extensions: Vec::new(),
            accepted_domains: Vec::new(),
            rejected_domains: Vec::new(),
            include_directories: Vec::new(),
            exclude_directories: Vec::new(),
            no_parent: false,
            no_host_directories: false,
            spider: false,
        }
    }
}

/// Recursive downloader
pub struct RecursiveDownloader {
    downloader: Downloader,
    config: RecursiveConfig,
    visited: HashSet<String>,
    queue: VecDeque<(String, usize)>, // (URL, depth)
    base_url: Option<String>, // Base URL for no_parent check
    broken_links: Vec<(String, u16)>, // (URL, status_code) for tracking broken links
}

impl RecursiveDownloader {
    pub fn new(download_config: DownloadConfig, recursive_config: RecursiveConfig) -> Result<Self> {
        Ok(Self {
            downloader: Downloader::new(download_config)?,
            config: recursive_config,
            visited: HashSet::new(),
            queue: VecDeque::new(),
            base_url: None,
            broken_links: Vec::new(),
        })
    }

    /// Get the list of broken links encountered during spider mode
    pub fn broken_links(&self) -> &[(String, u16)] {
        &self.broken_links
    }

    /// Start recursive download from a URL
    pub async fn download_recursive(
        &mut self,
        start_url: &str,
        output_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        let mut downloaded_files = Vec::new();

        // Set base URL for no_parent check
        self.base_url = Some(start_url.to_string());

        // Add starting URL to queue
        self.queue.push_back((start_url.to_string(), 0));

        while let Some((url, depth)) = self.queue.pop_front() {
            // Skip if already visited
            if self.visited.contains(&url) {
                continue;
            }

            // Skip if max depth exceeded
            if self.config.max_depth > 0 && depth >= self.config.max_depth {
                continue;
            }

            // Skip if URL doesn't match filters
            // Note: Pass depth to should_download so it can handle --https-only correctly
            // (starting URL is allowed even if HTTP, but extracted links are filtered)
            if !self.should_download(&url, depth)? {
                continue;
            }

            // Mark as visited
            self.visited.insert(url.clone());

            // Download the file
            let file_path = self.download_and_save(&url, output_dir, depth).await?;
            downloaded_files.push(file_path.clone());

            // Parse HTML and extract links if this is an HTML file/URL
            // In spider mode, we always try to extract links from HTML content
            // In normal mode, check if saved file is HTML
            let should_extract_links = if self.config.spider {
                // In spider mode, check if URL points to HTML content
                self.is_html_url(&url).await
            } else {
                // In normal mode, check if saved file is HTML
                self.is_html_file(&file_path)
            };

            if should_extract_links {
                let links = self.extract_links(&file_path, &url).await?;

                // Add links to queue
                for link in links {
                    if !self.visited.contains(&link) {
                        self.queue.push_back((link, depth + 1));
                    }
                }
            }
        }

        Ok(downloaded_files)
    }

    /// Check if URL should be downloaded
    fn should_download(&self, url: &str, depth: usize) -> Result<bool> {
        let parsed_url = Url::parse(url)
            .map_err(|e| Error::ConfigError(format!("Invalid URL: {}", e)))?;

        // Check HTTPS-only mode
        // Note: --https-only only applies to extracted links (depth > 0), not the starting URL
        // This matches GNU wget behavior: you can start with HTTP but only follow HTTPS links
        if self.downloader.get_client().config().https_only && depth > 0 {
            if parsed_url.scheme() != "https" {
                return Ok(false);
            }
        }

        let domain = parsed_url
            .host_str()
            .ok_or_else(|| Error::ConfigError("URL has no host".to_string()))?;

        // Check domain filters
        if !self.config.accepted_domains.is_empty() {
            if !self.config.accepted_domains.iter().any(|d| domain.contains(d)) {
                return Ok(false);
            }
        }

        if self.config.rejected_domains.iter().any(|d| domain.contains(d)) {
            return Ok(false);
        }

        // Check extension filters
        let path = parsed_url.path();
        if let Some(extension) = Path::new(path).extension() {
            let ext = extension.to_string_lossy().to_lowercase();

            if !self.config.accept_extensions.is_empty() {
                if !self.config.accept_extensions.contains(&ext) {
                    return Ok(false);
                }
            }

            if self.config.reject_extensions.contains(&ext) {
                return Ok(false);
            }
        }

        // Check directory filters
        if !self.config.include_directories.is_empty() {
            if !self.config.include_directories.iter().any(|d| path.contains(d)) {
                return Ok(false);
            }
        }

        if self.config.exclude_directories.iter().any(|d| path.contains(d)) {
            return Ok(false);
        }

        // Check no_parent option
        if self.config.no_parent {
            if let Some(ref base_url_str) = self.base_url {
                let base_parsed = Url::parse(base_url_str)
                    .map_err(|e| Error::ConfigError(format!("Invalid base URL: {}", e)))?;

                // Check if URL ascends to parent directory
                // Both URLs must be on the same host
                if parsed_url.host() == base_parsed.host() {
                    let base_path = base_parsed.path();
                    let current_path = parsed_url.path();

                    // Extract directory portion of base path
                    // If base path ends with '/', it's already a directory
                    // Otherwise, get the parent directory
                    let base_dir = if base_path.ends_with('/') {
                        base_path
                    } else {
                        // Get parent directory (everything up to and including last '/')
                        match base_path.rfind('/') {
                            Some(pos) => &base_path[..=pos], // Include the trailing '/'
                            None => "/", // Root directory
                        }
                    };

                    // If current path doesn't start with base directory, it's ascending to parent
                    if !current_path.starts_with(base_dir) {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    /// Download and save a file (or just check in spider mode)
    async fn download_and_save(
        &mut self,
        url: &str,
        output_dir: &Path,
        _depth: usize,
    ) -> Result<PathBuf> {
        // In spider mode, just check if URL exists without downloading
        if self.config.spider {
            // Use download_to_memory to check the URL (won't save to disk)
            match self.downloader.download_to_memory(url).await {
                Ok(_) => {
                    // URL is accessible - return a dummy path
                    Ok(PathBuf::from("/dev/null"))
                }
                Err(e) => {
                    // Check if it's an HTTP error
                    if let crate::Error::InvalidStatus(status_code) = &e {
                        // Track broken link
                        self.broken_links.push((url.to_string(), *status_code));
                    }
                    // In spider mode, we don't fail on errors - just track them
                    Ok(PathBuf::from("/dev/null"))
                }
            }
        } else {
            // Normal mode - download and save
            // Generate local file path
            let local_path = self.url_to_local_path(url, output_dir)?;

            // Create parent directories
            if let Some(parent) = local_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            // Download to file
            self.downloader
                .download_to_file(url, local_path.clone())
                .await?;

            Ok(local_path)
        }
    }

    /// Convert URL to local file path
    fn url_to_local_path(&self, url: &str, output_dir: &Path) -> Result<PathBuf> {
        let parsed = Url::parse(url)
            .map_err(|e| Error::ConfigError(format!("Invalid URL: {}", e)))?;

        let mut path = output_dir.to_path_buf();

        // Add host directory (unless no_host_directories is set)
        if !self.config.no_host_directories {
            if let Some(host) = parsed.host_str() {
                path.push(host);
            }
        }

        // Add path components
        if let Some(segments) = parsed.path_segments() {
            for segment in segments {
                if !segment.is_empty() {
                    path.push(segment);
                }
            }
        }

        // If path ends with /, add index.html
        if path.is_dir() || url.ends_with('/') {
            path.push("index.html");
        }

        Ok(path)
    }

    /// Check if file is HTML
    fn is_html_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            matches!(ext.as_str(), "html" | "htm" | "xhtml")
        } else {
            false
        }
    }

    /// Check if URL points to HTML content (for spider mode)
    async fn is_html_url(&self, url: &str) -> bool {
        // Get metadata to check content type
        if let Ok(metadata) = self.downloader.get_client().get_metadata(url).await {
            if let Some(content_type) = metadata.content_type {
                return content_type.contains("text/html");
            }
        }
        // Default to checking URL extension
        url.ends_with(".html") || url.ends_with(".htm") || url.ends_with("/")
    }

    /// Check if HTML document has meta robots nofollow directive
    fn has_meta_robots_nofollow(&self, document: &Html) -> bool {
        // Parse meta tags with name="robots" (case-insensitive)
        if let Ok(selector) = Selector::parse("meta[name]") {
            for element in document.select(&selector) {
                if let Some(name) = element.value().attr("name") {
                    // Check if name attribute is "robots" (case-insensitive)
                    if name.eq_ignore_ascii_case("robots") {
                        if let Some(content) = element.value().attr("content") {
                            // Parse comma-separated values in content attribute
                            // Check for "nofollow" (case-insensitive)
                            let directives: Vec<&str> = content
                                .split(',')
                                .map(|s| s.trim())
                                .collect();

                            for directive in directives {
                                if directive.eq_ignore_ascii_case("nofollow") {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Extract links from HTML file (or URL in spider mode)
    async fn extract_links(&self, file_path: &Path, base_url: &str) -> Result<Vec<String>> {
        // In spider mode, fetch the content from URL instead of file
        let content = if self.config.spider {
            // Download HTML content to memory
            match self.downloader.download_to_memory(base_url).await {
                Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                Err(_) => return Ok(Vec::new()), // Can't extract links if download failed
            }
        } else {
            tokio::fs::read_to_string(file_path).await?
        };

        let document = Html::parse_document(&content);

        // Check for meta robots nofollow directive
        if self.has_meta_robots_nofollow(&document) {
            // Don't extract any links from pages with nofollow directive
            return Ok(Vec::new());
        }

        let mut links = Vec::new();

        // Extract from <a> tags
        if let Ok(selector) = Selector::parse("a[href]") {
            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    if let Ok(absolute_url) = self.resolve_url(base_url, href) {
                        links.push(absolute_url);
                    }
                }
            }
        }

        // Extract page requisites if enabled
        if self.config.page_requisites {
            // Images
            if let Ok(selector) = Selector::parse("img[src]") {
                for element in document.select(&selector) {
                    if let Some(src) = element.value().attr("src") {
                        if let Ok(absolute_url) = self.resolve_url(base_url, src) {
                            links.push(absolute_url);
                        }
                    }
                }
            }

            // CSS
            if let Ok(selector) = Selector::parse("link[rel=stylesheet][href]") {
                for element in document.select(&selector) {
                    if let Some(href) = element.value().attr("href") {
                        if let Ok(absolute_url) = self.resolve_url(base_url, href) {
                            links.push(absolute_url);
                        }
                    }
                }
            }

            // Scripts
            if let Ok(selector) = Selector::parse("script[src]") {
                for element in document.select(&selector) {
                    if let Some(src) = element.value().attr("src") {
                        if let Ok(absolute_url) = self.resolve_url(base_url, src) {
                            links.push(absolute_url);
                        }
                    }
                }
            }
        }

        Ok(links)
    }

    /// Resolve relative URL to absolute
    fn resolve_url(&self, base: &str, relative: &str) -> Result<String> {
        let base_url = Url::parse(base)
            .map_err(|e| Error::ConfigError(format!("Invalid base URL: {}", e)))?;

        // Skip data: URLs and javascript: URLs
        if relative.starts_with("data:") || relative.starts_with("javascript:") {
            return Err(Error::ConfigError("Skipping non-HTTP URL".to_string()));
        }

        let absolute = base_url
            .join(relative)
            .map_err(|e| Error::ConfigError(format!("Failed to resolve URL: {}", e)))?;

        // Check if we should follow this link
        if self.config.relative_only {
            // Only follow if same host
            if base_url.host() != absolute.host() {
                return Err(Error::ConfigError("Skipping external link".to_string()));
            }
        }

        if !self.config.span_hosts {
            // Only follow if same host
            if base_url.host() != absolute.host() {
                return Err(Error::ConfigError("Skipping external host".to_string()));
            }
        }

        Ok(absolute.to_string())
    }
}
