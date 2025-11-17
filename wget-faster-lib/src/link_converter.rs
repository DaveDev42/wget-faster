/// Link conversion for making downloaded HTML/CSS suitable for local viewing
///
/// This module implements the wget -k (--convert-links) functionality:
/// - Converts absolute URLs to relative URLs in HTML and CSS files
/// - Updates href/src attributes in HTML
/// - Updates @import and `url()` in CSS
/// - Handles backup of original files with -K flag
use crate::{Error, Result};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use url::Url;

/// Link converter for making downloaded files suitable for local viewing
pub struct LinkConverter {
    /// Map of original URL to local file path
    url_to_path: HashMap<String, PathBuf>,

    /// Base directory for all downloads
    base_dir: PathBuf,

    /// Whether to backup original files before conversion
    backup_converted: bool,
}

impl LinkConverter {
    /// Create a new link converter
    pub fn new(base_dir: PathBuf, backup_converted: bool) -> Self {
        Self {
            url_to_path: HashMap::new(),
            base_dir,
            backup_converted,
        }
    }

    /// Register a downloaded file (maps URL to local path)
    pub fn register_file(&mut self, url: &str, path: PathBuf) {
        // Normalize URL (remove fragment)
        let normalized_url = if let Ok(parsed) = Url::parse(url) {
            let mut normalized = parsed.clone();
            normalized.set_fragment(None);
            normalized.to_string()
        } else {
            url.to_string()
        };

        self.url_to_path.insert(normalized_url, path);
    }

    /// Convert links in all registered HTML and CSS files
    pub async fn convert_all_links(&self) -> Result<()> {
        for (url, path) in &self.url_to_path {
            if self.is_html_file(path) {
                self.convert_html_file(path, url).await?;
            } else if self.is_css_file(path) {
                self.convert_css_file(path, url).await?;
            }
        }
        Ok(())
    }

    /// Check if file is HTML based on extension
    fn is_html_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            matches!(ext.as_str(), "html" | "htm" | "xhtml")
        } else {
            false
        }
    }

    /// Check if file is CSS based on extension
    fn is_css_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            ext == "css"
        } else {
            false
        }
    }

    /// Backup a file by copying it to .orig
    async fn backup_file(&self, path: &Path) -> Result<()> {
        if !self.backup_converted {
            return Ok(());
        }

        // Create backup path by replacing extension with .orig
        // For files like "index.php.html", we want "index.php.orig" not "index.php.html.orig"
        // This matches GNU wget's behavior: backup the original filename before -E extension
        let backup_path = if let Some(stem) = path.file_stem() {
            path.with_file_name(format!("{}.orig", stem.to_string_lossy()))
        } else {
            path.with_extension("orig")
        };

        tokio::fs::copy(path, backup_path)
            .await
            .map_err(Error::IoError)?;

        Ok(())
    }

    /// Convert links in an HTML file
    async fn convert_html_file(&self, path: &Path, base_url: &str) -> Result<()> {
        // Read HTML content
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(Error::IoError)?;

        // Convert links
        let converted = self.convert_html_content(&content, base_url)?;

        // Only backup and save if content actually changed (GNU wget behavior)
        // If no links were converted, don't create .orig file
        if converted != content {
            // Backup original file before writing converted version
            self.backup_file(path).await?;

            // Write converted content back
            tokio::fs::write(path, converted)
                .await
                .map_err(Error::IoError)?;

            tracing::debug!(path = %path.display(), "Links converted and file updated");
        } else {
            tracing::debug!(path = %path.display(), "No link conversions needed");
        }

        Ok(())
    }

    /// Convert links in HTML content
    fn convert_html_content(&self, html: &str, base_url: &str) -> Result<String> {
        let document = Html::parse_document(html);
        let mut result = html.to_string();

        // Parse base URL for resolving relative URLs
        let base = Url::parse(base_url)
            .map_err(|e| Error::ConfigError(format!("Invalid base URL: {e}")))?;

        // Convert <a href="...">
        if let Ok(selector) = Selector::parse("a[href]") {
            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    if let Some(new_href) = self.convert_url_to_relative(&base, href) {
                        result = result
                            .replace(&format!("href=\"{href}\""), &format!("href=\"{new_href}\""));
                    }
                }
            }
        }

        // Convert <img src="...">
        if let Ok(selector) = Selector::parse("img[src]") {
            for element in document.select(&selector) {
                if let Some(src) = element.value().attr("src") {
                    if let Some(new_src) = self.convert_url_to_relative(&base, src) {
                        result = result
                            .replace(&format!("src=\"{src}\""), &format!("src=\"{new_src}\""));
                    }
                }
            }
        }

        // Convert <link href="..."> (CSS, etc.)
        if let Ok(selector) = Selector::parse("link[href]") {
            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    if let Some(new_href) = self.convert_url_to_relative(&base, href) {
                        result = result
                            .replace(&format!("href=\"{href}\""), &format!("href=\"{new_href}\""));
                    }
                }
            }
        }

        // Convert <script src="...">
        if let Ok(selector) = Selector::parse("script[src]") {
            for element in document.select(&selector) {
                if let Some(src) = element.value().attr("src") {
                    if let Some(new_src) = self.convert_url_to_relative(&base, src) {
                        result = result
                            .replace(&format!("src=\"{src}\""), &format!("src=\"{new_src}\""));
                    }
                }
            }
        }

        Ok(result)
    }

    /// Convert links in a CSS file
    async fn convert_css_file(&self, path: &Path, base_url: &str) -> Result<()> {
        // Backup original file if requested
        self.backup_file(path).await?;

        // Read CSS content
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(Error::IoError)?;

        // Convert links
        let converted = self.convert_css_content(&content, base_url)?;

        // Write converted content back
        tokio::fs::write(path, converted)
            .await
            .map_err(Error::IoError)?;

        Ok(())
    }

    /// Convert links in CSS content (`url()` and @import)
    fn convert_css_content(&self, css: &str, base_url: &str) -> Result<String> {
        let base = Url::parse(base_url)
            .map_err(|e| Error::ConfigError(format!("Invalid base URL: {e}")))?;

        let mut result = css.to_string();

        // Find all url() references
        // Match url("..."), url('...'), and url(...)
        let url_regex = regex::Regex::new(r#"url\s*\(\s*['"]?([^'")]+)['"]?\s*\)"#)
            .map_err(|e| Error::ConfigError(format!("Regex error: {e}")))?;

        for cap in url_regex.captures_iter(css) {
            if let Some(url_match) = cap.get(1) {
                let original_url = url_match.as_str();
                if let Some(new_url) = self.convert_url_to_relative(&base, original_url) {
                    result =
                        result.replace(&format!("url({original_url})"), &format!("url({new_url})"));
                    result = result.replace(
                        &format!("url(\"{original_url}\")"),
                        &format!("url(\"{new_url}\")"),
                    );
                    result = result
                        .replace(&format!("url('{original_url}')"), &format!("url('{new_url}')"));
                }
            }
        }

        // Find all @import references
        let import_regex = regex::Regex::new(r#"@import\s+['"]([^'"]+)['"]"#)
            .map_err(|e| Error::ConfigError(format!("Regex error: {e}")))?;

        for cap in import_regex.captures_iter(css) {
            if let Some(url_match) = cap.get(1) {
                let original_url = url_match.as_str();
                if let Some(new_url) = self.convert_url_to_relative(&base, original_url) {
                    result = result.replace(
                        &format!("@import \"{original_url}\""),
                        &format!("@import \"{new_url}\""),
                    );
                    result = result.replace(
                        &format!("@import '{original_url}'"),
                        &format!("@import '{new_url}'"),
                    );
                }
            }
        }

        Ok(result)
    }

    /// Convert an absolute URL to a relative path if the file was downloaded
    fn convert_url_to_relative(&self, base: &Url, url_str: &str) -> Option<String> {
        // Skip data: URLs, javascript:, mailto:, etc.
        if url_str.starts_with("data:")
            || url_str.starts_with("javascript:")
            || url_str.starts_with("mailto:")
            || url_str.starts_with('#')
        {
            return None;
        }

        // Resolve the URL relative to base
        let absolute_url = if let Ok(parsed) = Url::parse(url_str) {
            // Already absolute
            parsed
        } else {
            // Relative URL - resolve against base
            match base.join(url_str) {
                Ok(resolved) => resolved,
                Err(_) => return None,
            }
        };

        // Normalize (remove fragment)
        let mut normalized = absolute_url.clone();
        normalized.set_fragment(None);
        let normalized_str = normalized.to_string();

        // Check if we downloaded this file
        if let Some(target_path) = self.url_to_path.get(&normalized_str) {
            // Convert absolute path to relative path from base directory
            if let Ok(relative) = target_path.strip_prefix(&self.base_dir) {
                let relative_str = relative.to_string_lossy();

                // GNU wget compatibility: add "./" prefix if the filename contains ':'
                // and has no directory separators (basedirs == 0)
                // This prevents filenames like "site;sub:.html" from being misinterpreted
                // Reference: GNU wget's construct_relative() in src/convert.c
                let needs_prefix = !relative_str.contains('/') && relative_str.contains(':');

                if needs_prefix {
                    return Some(format!("./{}", relative_str));
                } else {
                    return Some(relative_str.to_string());
                }
            }
        }

        None
    }
}
