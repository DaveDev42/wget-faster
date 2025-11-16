/// Recursive download functionality for downloading entire websites
use crate::{DownloadConfig, Downloader, Error, LinkConverter, Result};
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet, VecDeque};
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

    /// Backup original files before converting (with -K flag)
    pub backup_converted: bool,

    /// Adjust file extensions (.html for HTML/CSS files) - used with -E flag
    pub adjust_extension: bool,

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

    /// Log rejected URLs to a file
    pub rejected_log: Option<PathBuf>,

    /// Don't create directories (save all files in output directory)
    pub no_directories: bool,
}

impl Default for RecursiveConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            span_hosts: false,
            relative_only: false,
            convert_links: false,
            backup_converted: false,
            adjust_extension: false,
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
            rejected_log: None,
            no_directories: false,
        }
    }
}

/// Recursive downloader
pub struct RecursiveDownloader {
    downloader: Downloader,
    config: RecursiveConfig,
    visited: HashSet<String>,
    queue: VecDeque<(String, usize, Option<String>)>, // (URL, depth, parent_url)
    base_url: Option<String>,                         // Base URL for no_parent check
    broken_links: Vec<(String, u16)>, // (URL, status_code) for tracking broken links
    link_converter: Option<LinkConverter>, // Link converter for -k flag
    rejected_urls: Vec<(String, String, Option<String>)>, // (URL, reason, parent_url) for tracking rejected URLs
    robots_cache: HashMap<String, Option<crate::robots::RobotsTxt>>, // Cache of robots.txt per host (None if not found/failed)
    spider_content_cache: HashMap<String, Option<String>>, // Cache of HTML content in spider mode (None if download failed)
}

impl RecursiveDownloader {
    /// Create a new recursive downloader
    ///
    /// # Arguments
    ///
    /// * `download_config` - Configuration for individual file downloads
    /// * `recursive_config` - Configuration for recursive behavior (depth, filters, etc.)
    ///
    /// # Returns
    ///
    /// Returns `Ok(RecursiveDownloader)` on success, or `Err` if the downloader
    /// configuration fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wget_faster_lib::{RecursiveDownloader, DownloadConfig, RecursiveConfig};
    ///
    /// let download_config = DownloadConfig::default();
    /// let mut recursive_config = RecursiveConfig::default();
    /// recursive_config.max_depth = 2;  // Only follow links 2 levels deep
    /// recursive_config.page_requisites = true;  // Download CSS, images, JS
    ///
    /// let downloader = RecursiveDownloader::new(download_config, recursive_config)?;
    /// # Ok::<(), wget_faster_lib::Error>(())
    /// ```
    pub fn new(
        mut download_config: DownloadConfig,
        recursive_config: RecursiveConfig,
    ) -> Result<Self> {
        // Disable parallel downloads for NORMAL recursive mode to match GNU wget behavior
        // GNU wget doesn't send HEAD requests during normal recursive downloads
        // BUT in spider mode, GNU wget DOES send HEAD requests before GET
        // So we only disable parallel downloads (which triggers HEAD requests) in non-spider mode
        if !recursive_config.spider {
            download_config.parallel_chunks = 1;
            download_config.parallel_threshold = 0;
        }

        Ok(Self {
            downloader: Downloader::new(download_config)?,
            config: recursive_config,
            visited: HashSet::new(),
            queue: VecDeque::new(),
            base_url: None,
            broken_links: Vec::new(),
            link_converter: None,
            rejected_urls: Vec::new(),
            robots_cache: HashMap::new(),
            spider_content_cache: HashMap::new(),
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

        // Initialize link converter if convert_links is enabled
        if self.config.convert_links {
            self.link_converter =
                Some(LinkConverter::new(output_dir.to_path_buf(), self.config.backup_converted));
        }

        // Set base URL for no_parent check
        self.base_url = Some(start_url.to_string());

        // Add starting URL to queue (no parent URL)
        self.queue.push_back((start_url.to_string(), 0, None));

        while let Some((url, depth, parent_url)) = self.queue.pop_front() {
            // Skip if already visited (log as BLACKLIST - recursive loop)
            if self.visited.contains(&url) {
                // Log this as a rejection if it has a parent (i.e., it's a link from another page)
                // This prevents logging the starting URL when it's first queued
                if parent_url.is_some() {
                    self.log_rejected_url(
                        &url,
                        "Already visited (recursive loop)",
                        parent_url.as_deref(),
                    );
                }
                continue;
            }

            // Skip if max depth exceeded
            if self.config.max_depth > 0 && depth >= self.config.max_depth {
                continue;
            }

            // Skip if URL doesn't match filters
            // Note: Pass depth to should_download so it can handle --https-only correctly
            // (starting URL is allowed even if HTTP, but extracted links are filtered)
            match self
                .should_download(&url, depth, parent_url.as_deref(), output_dir)
                .await
            {
                Ok(true) => {
                    // URL passed all filters
                },
                Ok(false) => {
                    // URL was rejected - the reason was already logged
                    continue;
                },
                Err(e) => return Err(e),
            }

            // Mark as visited
            self.visited.insert(url.clone());

            // Download the file
            let file_path = self.download_and_save(&url, output_dir, depth).await?;

            // Register file with link converter if enabled
            if let Some(ref mut converter) = self.link_converter {
                converter.register_file(&url, file_path.clone());
            }

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

                // Add links to queue (with current URL as parent)
                // Note: We queue ALL links, even if already visited, so we can log them as rejected
                for link in links {
                    self.queue.push_back((link, depth + 1, Some(url.clone())));
                }
            }
        }

        // Convert links after all files are downloaded
        if let Some(ref converter) = self.link_converter {
            converter.convert_all_links().await?;
        }

        // Write rejected URLs to log file if configured
        if let Some(ref log_path) = self.config.rejected_log {
            if !self.rejected_urls.is_empty() {
                use tokio::io::AsyncWriteExt;
                let mut file = tokio::fs::File::create(log_path).await?;

                // Write CSV header
                file.write_all(b"REASON\tU_URL\tU_SCHEME\tU_HOST\tU_PORT\tU_PATH\tU_PARAMS\tU_QUERY\tU_FRAGMENT\tP_URL\tP_SCHEME\tP_HOST\tP_PORT\tP_PATH\tP_PARAMS\tP_QUERY\tP_FRAGMENT\n")
                    .await?;

                // Write rejected URLs in CSV format
                for (url, reason, parent_url) in &self.rejected_urls {
                    if let Ok(csv_line) =
                        self.format_rejected_url_csv(url, reason, parent_url.as_deref())
                    {
                        file.write_all(csv_line.as_bytes()).await?;
                        file.write_all(b"\n").await?;
                    }
                }
            }
        }

        Ok(downloaded_files)
    }

    /// Fetch and parse robots.txt for a given host
    async fn fetch_robots_txt(
        &mut self,
        host: &str,
        scheme: &str,
        port: Option<u16>,
        output_dir: &Path,
    ) -> Option<crate::robots::RobotsTxt> {
        // Check cache first
        let cache_key =
            format!("{}://{}{}", scheme, host, port.map(|p| format!(":{p}")).unwrap_or_default());

        if let Some(cached) = self.robots_cache.get(&cache_key) {
            return cached.clone();
        }

        // Build robots.txt URL
        let robots_url = if let Some(p) = port {
            format!("{scheme}://{host}:{p}/robots.txt")
        } else {
            format!("{scheme}://{host}/robots.txt")
        };

        // Try to fetch robots.txt
        // Use client().get() directly to avoid HEAD request (robots.txt doesn't need metadata)
        let robots_txt = match self
            .downloader
            .get_client()
            .client()
            .get(&robots_url)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                match response.bytes().await {
                    Ok(bytes) => {
                        // Parse the content
                        let content = String::from_utf8_lossy(&bytes);

                        // Save robots.txt to disk (unless in spider mode)
                        if !self.config.spider {
                            if let Ok(local_path) = self.url_to_local_path(&robots_url, output_dir)
                            {
                                // Create parent directories
                                if let Some(parent) = local_path.parent() {
                                    let _ = tokio::fs::create_dir_all(parent).await;
                                }
                                // Write the file
                                let _ = tokio::fs::write(&local_path, bytes.as_ref()).await;
                            }
                        }

                        Some(crate::robots::RobotsTxt::parse(&content))
                    },
                    Err(_) => None,
                }
            },
            _ => {
                // robots.txt not found or error - allow everything
                None
            },
        };

        // Cache the result
        self.robots_cache.insert(cache_key, robots_txt.clone());
        robots_txt
    }

    /// Check if URL should be downloaded
    async fn should_download(
        &mut self,
        url: &str,
        depth: usize,
        parent_url: Option<&str>,
        output_dir: &Path,
    ) -> Result<bool> {
        let parsed_url =
            Url::parse(url).map_err(|e| Error::ConfigError(format!("Invalid URL: {e}")))?;

        // Check HTTPS-only mode
        // Note: --https-only only applies to extracted links (depth > 0), not the starting URL
        // This matches GNU wget behavior: you can start with HTTP but only follow HTTPS links
        if self.downloader.get_client().config().https_only
            && depth > 0
            && parsed_url.scheme() != "https"
        {
            self.log_rejected_url(url, "Non-HTTPS URL rejected (HTTPS-only mode)", parent_url);
            return Ok(false);
        }

        let domain = parsed_url
            .host_str()
            .ok_or_else(|| Error::ConfigError("URL has no host".to_string()))?;

        // Check robots.txt (only for depth > 0, i.e., extracted links, not the starting URL)
        if depth > 0 {
            let scheme = parsed_url.scheme();
            let port = parsed_url.port();

            // Fetch robots.txt for this host
            if let Some(robots) = self
                .fetch_robots_txt(domain, scheme, port, output_dir)
                .await
            {
                let path = parsed_url.path();
                let user_agent = &self.downloader.get_client().config().user_agent;

                if !robots.is_allowed(path, user_agent) {
                    self.log_rejected_url(url, "Rejected by robots.txt rules", parent_url);
                    return Ok(false);
                }
            }
        }

        // Check span_hosts (only for extracted links, not starting URL)
        if !self.config.span_hosts && depth > 0 {
            // Get the base domain from the starting URL
            if let Some(ref base_url_str) = self.base_url {
                if let Ok(base_parsed) = Url::parse(base_url_str) {
                    if base_parsed.host() != parsed_url.host() {
                        self.log_rejected_url(
                            url,
                            &format!("Domain not in accepted list: {domain}"),
                            parent_url,
                        );
                        return Ok(false);
                    }
                }
            }
        }

        // Check domain filters
        if !self.config.accepted_domains.is_empty()
            && !self
                .config
                .accepted_domains
                .iter()
                .any(|d| domain.contains(d))
        {
            self.log_rejected_url(
                url,
                &format!("Domain not in accepted list: {domain}"),
                parent_url,
            );
            return Ok(false);
        }

        if self
            .config
            .rejected_domains
            .iter()
            .any(|d| domain.contains(d))
        {
            self.log_rejected_url(url, &format!("Domain in rejected list: {domain}"), parent_url);
            return Ok(false);
        }

        // Check extension filters
        let path = parsed_url.path();
        if let Some(extension) = Path::new(path).extension() {
            let ext = extension.to_string_lossy().to_lowercase();

            if !self.config.accept_extensions.is_empty()
                && !self.config.accept_extensions.contains(&ext)
            {
                self.log_rejected_url(
                    url,
                    &format!("Extension not in accepted list: {ext}"),
                    parent_url,
                );
                return Ok(false);
            }

            if self.config.reject_extensions.contains(&ext) {
                self.log_rejected_url(
                    url,
                    &format!("Extension in rejected list: {ext}"),
                    parent_url,
                );
                return Ok(false);
            }
        }

        // Check directory filters
        if !self.config.include_directories.is_empty()
            && !self
                .config
                .include_directories
                .iter()
                .any(|d| path.contains(d))
        {
            self.log_rejected_url(
                url,
                &format!("Directory not in include list: {path}"),
                parent_url,
            );
            return Ok(false);
        }

        if self
            .config
            .exclude_directories
            .iter()
            .any(|d| path.contains(d))
        {
            self.log_rejected_url(url, &format!("Directory in exclude list: {path}"), parent_url);
            return Ok(false);
        }

        // Check no_parent option
        if self.config.no_parent {
            if let Some(ref base_url_str) = self.base_url {
                let base_parsed = Url::parse(base_url_str)
                    .map_err(|e| Error::ConfigError(format!("Invalid base URL: {e}")))?;

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
                            None => "/",                     // Root directory
                        }
                    };

                    // If current path doesn't start with base directory, it's ascending to parent
                    if !current_path.starts_with(base_dir) {
                        self.log_rejected_url(
                            url,
                            "Ascends to parent directory (no-parent mode)",
                            parent_url,
                        );
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    /// Log a rejected URL with a reason (if `rejected_log` is enabled)
    fn log_rejected_url(&mut self, url: &str, reason: &str, parent_url: Option<&str>) {
        if self.config.rejected_log.is_some() {
            self.rejected_urls.push((
                url.to_string(),
                reason.to_string(),
                parent_url.map(|s| s.to_string()),
            ));
        }
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
            // Also cache the content for later use in extract_links
            match self.downloader.download_to_memory(url).await {
                Ok(bytes) => {
                    // URL is accessible - cache the content as string
                    let content = String::from_utf8_lossy(&bytes).to_string();
                    self.spider_content_cache
                        .insert(url.to_string(), Some(content));
                    // Return a dummy path
                    Ok(PathBuf::from("/dev/null"))
                },
                Err(e) => {
                    // Check if it's an HTTP error
                    if let crate::Error::InvalidStatus(status_code) = &e {
                        // Track broken link
                        self.broken_links.push((url.to_string(), *status_code));
                    }
                    // Cache the failure (None) so we don't try to download again
                    self.spider_content_cache.insert(url.to_string(), None);
                    // In spider mode, we don't fail on errors - just track them
                    Ok(PathBuf::from("/dev/null"))
                },
            }
        } else {
            // Normal mode - download and save
            // Generate local file path
            let local_path = self.url_to_local_path(url, output_dir)?;

            // Create parent directories
            // Handle the case where a file exists with the same name as a directory we need
            // This can happen with redirects: /directory (saved as file) -> /directory/ (needs directory)
            if let Some(parent) = local_path.parent() {
                match tokio::fs::create_dir_all(parent).await {
                    Ok(()) => {},
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                        // Check if parent exists as a file (not a directory)
                        if let Ok(metadata) = tokio::fs::metadata(parent).await {
                            if metadata.is_file() {
                                // Parent exists as a file - remove it and create directory
                                tracing::warn!(
                                    path = %parent.display(),
                                    "Removing file to create directory (likely due to redirect from /path to /path/)"
                                );
                                tokio::fs::remove_file(parent).await?;
                                tokio::fs::create_dir_all(parent).await?;
                            }
                            // If it's already a directory, we're good
                        } else {
                            // Metadata failed - propagate original error
                            return Err(e.into());
                        }
                    },
                    Err(e) => return Err(e.into()),
                }
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
        let parsed =
            Url::parse(url).map_err(|e| Error::ConfigError(format!("Invalid URL: {e}")))?;

        let mut path = output_dir.to_path_buf();

        // If no_directories is set, just use the filename without any directory structure
        if self.config.no_directories {
            // Extract just the filename from the URL
            let filename = parsed
                .path_segments()
                .and_then(|mut segments| segments.next_back())
                .filter(|name| !name.is_empty())
                .unwrap_or("index.html");

            path.push(filename);
        } else {
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
        }

        // If path ends with /, add index.html
        if path.is_dir() || url.ends_with('/') {
            path.push("index.html");
        }

        // Adjust extension if requested (-E flag)
        // Add .html extension to files that don't have one but are HTML/CSS content
        if self.config.adjust_extension {
            let current_ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

            // If the file doesn't already have .html extension, add it
            // This matches wget -E behavior: file.php -> file.php.html
            if !current_ext.is_empty() && current_ext != "html" && current_ext != "htm" {
                // Only add .html if it looks like server-side script or no extension
                // Common server-side extensions: php, asp, aspx, jsp, cgi, pl
                if matches!(
                    current_ext,
                    "php" | "asp" | "aspx" | "jsp" | "cgi" | "pl" | "py" | "rb"
                ) {
                    path.set_extension(format!("{current_ext}.html"));
                }
            }
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
        // Check URL extension first (fast path - avoids HEAD request)
        // This matches GNU wget behavior: only send HEAD if content type is uncertain
        if url.ends_with(".html") || url.ends_with(".htm") || url.ends_with('/') {
            return true;
        }

        // Non-HTML extensions (skip HEAD request)
        if url.ends_with(".jpg")
            || url.ends_with(".jpeg")
            || url.ends_with(".png")
            || url.ends_with(".gif")
            || url.ends_with(".webp")
            || url.ends_with(".css")
            || url.ends_with(".js")
            || url.ends_with(".ico")
            || url.ends_with(".pdf")
            || url.ends_with(".zip")
            || url.ends_with(".tar")
            || url.ends_with(".gz")
        {
            return false;
        }

        // Uncertain - only NOW send HEAD request to check content type
        if let Ok(metadata) = self.downloader.get_client().get_metadata(url).await {
            if let Some(content_type) = metadata.content_type {
                return content_type.contains("text/html");
            }
        }

        // Default: treat as HTML if uncertain (matches wget behavior)
        true
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
                            let directives: Vec<&str> = content.split(',').map(str::trim).collect();

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
            // Check cache first - content was already downloaded in download_and_save()
            if let Some(cached) = self.spider_content_cache.get(base_url) {
                if let Some(content) = cached {
                    content.clone()
                } else {
                    return Ok(Vec::new()); // Download failed, already tracked
                }
            } else {
                // Cache miss (shouldn't happen in normal flow, but handle gracefully)
                match self.downloader.download_to_memory(base_url).await {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                    Err(_) => return Ok(Vec::new()), // Can't extract links if download failed
                }
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

            // Image srcset attribute (responsive images)
            // Format: "url1, url2 descriptor, url3 descriptor"
            // Example: "image1.png, image2.png 150w, image3.png 100x"
            if let Ok(selector) = Selector::parse("img[srcset]") {
                for element in document.select(&selector) {
                    if let Some(srcset) = element.value().attr("srcset") {
                        for entry in srcset.split(',') {
                            // Split on whitespace and take first part (URL)
                            // The rest are descriptors (150w, 2x, etc.)
                            if let Some(url) = entry.trim().split_whitespace().next() {
                                if let Ok(absolute_url) = self.resolve_url(base_url, url) {
                                    links.push(absolute_url);
                                }
                            }
                        }
                    }
                }
            }

            // Source srcset attribute (picture element)
            if let Ok(selector) = Selector::parse("source[srcset]") {
                for element in document.select(&selector) {
                    if let Some(srcset) = element.value().attr("srcset") {
                        for entry in srcset.split(',') {
                            if let Some(url) = entry.trim().split_whitespace().next() {
                                if let Ok(absolute_url) = self.resolve_url(base_url, url) {
                                    links.push(absolute_url);
                                }
                            }
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
        let base_url =
            Url::parse(base).map_err(|e| Error::ConfigError(format!("Invalid base URL: {e}")))?;

        // Skip data: URLs and javascript: URLs
        if relative.starts_with("data:") || relative.starts_with("javascript:") {
            return Err(Error::ConfigError("Skipping non-HTTP URL".to_string()));
        }

        let absolute = base_url
            .join(relative)
            .map_err(|e| Error::ConfigError(format!("Failed to resolve URL: {e}")))?;

        // Don't filter based on span_hosts here - let should_download() handle it
        // so rejected URLs can be logged properly
        // Just return the resolved URL
        Ok(absolute.to_string())
    }

    /// Format a rejected URL as a CSV line
    /// Format: REASON\tU_URL\tU_SCHEME\tU_HOST\tU_PORT\tU_PATH\t...
    fn format_rejected_url_csv(
        &self,
        url: &str,
        reason: &str,
        parent_url: Option<&str>,
    ) -> Result<String> {
        use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

        // Custom encoding set that encodes : but preserves /
        // This matches wget's CSV format: http%3A//host/path
        const URL_ENCODE: &AsciiSet = &CONTROLS.add(b':');

        let parsed =
            Url::parse(url).map_err(|e| Error::ConfigError(format!("Invalid URL: {e}")))?;

        // Map rejection reason to CSV reason code
        let csv_reason = if reason.contains("robots.txt") {
            "ROBOTS"
        } else if reason.contains("Domain in rejected list")
            || reason.contains("Domain not in accepted list")
        {
            "SPANNEDHOST"
        } else if reason.contains("Extension") || reason.contains("Directory") {
            "BLACKLIST"
        } else if reason.contains("parent directory") {
            "BLACKLIST"
        } else if reason.contains("Already visited") {
            "BLACKLIST" // Recursive loops
        } else {
            "BLACKLIST" // Default to BLACKLIST for unknown reasons
        };

        // URL-encode the URLs (encode : to match wget format)
        let encoded_url = utf8_percent_encode(url, URL_ENCODE).to_string();

        // Get scheme type
        let scheme = if parsed.scheme() == "https" {
            "SCHEME_HTTPS"
        } else if parsed.scheme() == "http" {
            "SCHEME_HTTP"
        } else if parsed.scheme() == "ftp" {
            "SCHEME_FTP"
        } else {
            "SCHEME_HTTP" // Default
        };

        let host = parsed.host_str().unwrap_or("");
        let port = parsed.port().unwrap_or_else(|| match parsed.scheme() {
            "https" => 443,
            "http" => 80,
            "ftp" => 21,
            _ => 80,
        });
        let path = parsed.path().trim_start_matches('/');
        let query = parsed.query().unwrap_or("");
        let fragment = parsed.fragment().unwrap_or("");

        // Format parent URL columns if parent_url is provided
        let parent_cols = if let Some(p_url) = parent_url {
            if let Ok(p_parsed) = Url::parse(p_url) {
                let p_encoded = utf8_percent_encode(p_url, URL_ENCODE).to_string();
                let p_scheme = if p_parsed.scheme() == "https" {
                    "SCHEME_HTTPS"
                } else if p_parsed.scheme() == "http" {
                    "SCHEME_HTTP"
                } else if p_parsed.scheme() == "ftp" {
                    "SCHEME_FTP"
                } else {
                    "SCHEME_HTTP"
                };
                let p_host = p_parsed.host_str().unwrap_or("");
                let p_port = p_parsed.port().unwrap_or_else(|| match p_parsed.scheme() {
                    "https" => 443,
                    "http" => 80,
                    "ftp" => 21,
                    _ => 80,
                });
                let p_path = p_parsed.path().trim_start_matches('/');
                let p_query = p_parsed.query().unwrap_or("");
                let p_fragment = p_parsed.fragment().unwrap_or("");

                format!(
                    "{}\t{}\t{}\t{}\t{}\t\t{}\t{}",
                    p_encoded, p_scheme, p_host, p_port, p_path, p_query, p_fragment
                )
            } else {
                "\t\t\t\t\t\t\t".to_string()
            }
        } else {
            "\t\t\t\t\t\t\t".to_string()
        };

        // Format CSV line (tab-separated)
        Ok(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t\t{}\t{}\t{}",
            csv_reason, encoded_url, scheme, host, port, path, query, fragment, parent_cols
        ))
    }
}
