use crate::{Error, Result, DownloadConfig};
use reqwest::{Client, ClientBuilder, header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT, ACCEPT_ENCODING}};
use std::time::Duration;

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    config: DownloadConfig,
}

impl HttpClient {
    pub fn new(config: DownloadConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();

        // Set user agent
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&config.user_agent)?
        );

        // Set compression if enabled
        if config.enable_compression {
            headers.insert(
                ACCEPT_ENCODING,
                HeaderValue::from_static("gzip, deflate, br")
            );
        }

        // Add custom headers
        for (key, value) in &config.headers {
            let header_name = HeaderName::from_bytes(key.as_bytes())?;
            let header_value = HeaderValue::from_str(value)?;
            headers.insert(header_name, header_value);
        }

        let mut builder = ClientBuilder::new()
            .default_headers(headers)
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .pool_max_idle_per_host(config.parallel_chunks);

        // Configure redirects
        if config.follow_redirects {
            builder = builder.redirect(reqwest::redirect::Policy::limited(config.max_redirects));
        } else {
            builder = builder.redirect(reqwest::redirect::Policy::none());
        }

        // Configure SSL/TLS
        builder = builder.danger_accept_invalid_certs(!config.verify_ssl);

        // Configure proxy
        // Note: no_proxy filtering is done per-request basis, not at client level
        // reqwest doesn't support dynamic no_proxy, so we configure the proxy here
        // and handle no_proxy logic in the request builder
        if let Some(proxy_config) = &config.proxy {
            let mut proxy = reqwest::Proxy::all(&proxy_config.url)
                .map_err(|e| Error::ConfigError(format!("Invalid proxy URL: {}", e)))?;

            if let Some((username, password)) = &proxy_config.auth {
                proxy = proxy.basic_auth(username, password);
            }

            // Set no_proxy list if provided
            if !proxy_config.no_proxy.is_empty() {
                // reqwest supports NO_PROXY via environment variable parsing
                // But we need to handle it manually for consistency with wget
                let no_proxy = reqwest::NoProxy::from_string(
                    &proxy_config.no_proxy.join(",")
                );
                proxy = proxy.no_proxy(no_proxy);
            }

            builder = builder.proxy(proxy);
        }

        // Configure authentication
        // Note: Basic auth will be added per-request
        // Digest auth is handled automatically by reqwest

        // Configure cookies
        if config.enable_cookies {
            builder = builder.cookie_store(true);

            // Load cookies from file if specified
            if let Some(ref cookie_file) = config.cookie_file {
                if cookie_file.exists() {
                    // Note: reqwest doesn't provide a way to load cookies from file directly
                    // We'll handle this at a higher level
                }
            }
        }

        // Configure certificates
        if let Some(ca_cert_path) = &config.ca_cert {
            let cert = std::fs::read(ca_cert_path)?;
            let cert = reqwest::Certificate::from_pem(&cert)
                .map_err(|e| Error::ConfigError(format!("Invalid CA certificate: {}", e)))?;
            builder = builder.add_root_certificate(cert);
        }

        if let Some(client_cert_path) = &config.client_cert {
            let cert = std::fs::read(client_cert_path)?;
            let identity = reqwest::Identity::from_pem(&cert)
                .map_err(|e| Error::ConfigError(format!("Invalid client certificate: {}", e)))?;
            builder = builder.identity(identity);
        }

        let client = builder
            .build()
            .map_err(|e| Error::ConfigError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn config(&self) -> &DownloadConfig {
        &self.config
    }

    /// Check if server supports range requests
    pub async fn supports_range(&self, url: &str) -> Result<bool> {
        let response = self.client.head(url).send().await?;

        Ok(response
            .headers()
            .get(reqwest::header::ACCEPT_RANGES)
            .and_then(|v| v.to_str().ok())
            .map(|v| v != "none")
            .unwrap_or(false))
    }

    /// Get content length from HEAD request
    pub async fn get_content_length(&self, url: &str) -> Result<Option<u64>> {
        let response = self.client.head(url).send().await?;

        Ok(response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok()))
    }

    /// Get metadata about the resource (supports range, content length, etc.)
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch metadata for
    /// * `if_modified_since` - Optional timestamp for conditional GET (If-Modified-Since header)
    pub async fn get_metadata(&self, url: &str) -> Result<ResourceMetadata> {
        self.get_metadata_conditional(url, None).await
    }

    /// Get metadata about the resource with optional If-Modified-Since header
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch metadata for
    /// * `if_modified_since` - Optional timestamp for conditional GET (If-Modified-Since header)
    pub async fn get_metadata_conditional(
        &self,
        url: &str,
        if_modified_since: Option<std::time::SystemTime>,
    ) -> Result<ResourceMetadata> {
        // Build HEAD request with optional If-Modified-Since header
        let mut request = self.client.head(url);

        // Add If-Modified-Since header if provided
        if let Some(time) = if_modified_since {
            let http_date = httpdate::fmt_http_date(time);
            request = request.header(reqwest::header::IF_MODIFIED_SINCE, http_date);
        }

        // Add authentication if configured and auth_no_challenge is set (preemptive auth)
        if self.config.auth_no_challenge {
            if let Some(ref auth) = self.config.auth {
                request = request.basic_auth(&auth.username, Some(&auth.password));
            }
        }

        let response = request.send().await?;
        let status_code = response.status().as_u16();

        // Handle authentication challenges (401/407)
        // If we have credentials but didn't send them preemptively, retry with auth
        if (status_code == 401 || status_code == 407) && !self.config.auth_no_challenge {
            // Try configured auth first, then .netrc
            let auth = if let Some(ref auth) = self.config.auth {
                Some(auth.clone())
            } else {
                // Try .netrc file
                match crate::netrc::Netrc::from_default_location() {
                    Ok(Some(netrc)) => {
                        // Extract hostname from URL
                        if let Ok(parsed) = url::Url::parse(url) {
                            if let Some(host) = parsed.host_str() {
                                netrc.get(host)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            };

            if let Some(auth) = auth {
                // Retry HEAD request with authentication
                let mut retry_request = self.client.head(url)
                    .basic_auth(&auth.username, Some(&auth.password));

                // Re-add If-Modified-Since header if it was present
                if let Some(time) = if_modified_since {
                    let http_date = httpdate::fmt_http_date(time);
                    retry_request = retry_request.header(reqwest::header::IF_MODIFIED_SINCE, http_date);
                }

                let retry_response = retry_request.send().await?;

                // If still unauthorized, return error (will be captured in metadata)
                // Don't fail here - let the caller handle it
                return Self::extract_metadata(retry_response).await;
            }
        }

        Self::extract_metadata(response).await
    }

    /// Extract metadata from a HEAD response
    async fn extract_metadata(response: reqwest::Response) -> Result<ResourceMetadata> {
        let supports_range = response
            .headers()
            .get(reqwest::header::ACCEPT_RANGES)
            .and_then(|v| v.to_str().ok())
            .map(|v| v != "none")
            .unwrap_or(false);

        let content_length = response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        let last_modified = response
            .headers()
            .get(reqwest::header::LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let etag = response
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let content_disposition = response
            .headers()
            .get(reqwest::header::CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let status_code = response.status().as_u16();
        let headers = response.headers().clone();

        Ok(ResourceMetadata {
            supports_range,
            content_length,
            last_modified,
            etag,
            content_type,
            content_disposition,
            status_code,
            headers,
        })
    }
}

/// Metadata about a remote resource
///
/// Contains information extracted from HTTP response headers, including
/// support for range requests, content information, and raw headers.
#[derive(Debug, Clone)]
pub struct ResourceMetadata {
    /// Whether the server supports HTTP Range requests
    pub supports_range: bool,

    /// Content length in bytes
    pub content_length: Option<u64>,

    /// Last-Modified header value
    pub last_modified: Option<String>,

    /// ETag header value
    pub etag: Option<String>,

    /// Content-Type header value
    pub content_type: Option<String>,

    /// Content-Disposition header value
    pub content_disposition: Option<String>,

    /// HTTP status code
    pub status_code: u16,

    /// Raw response headers (for --server-response display)
    pub headers: HeaderMap,
}

impl ResourceMetadata {
    /// Format headers for display (wget --server-response style)
    ///
    /// Returns a string with all HTTP headers formatted as "Header-Name: value"
    pub fn format_headers(&self) -> String {
        let mut output = format!("HTTP/1.1 {} {}\n",
            self.status_code,
            status_text(self.status_code)
        );

        for (name, value) in &self.headers {
            if let Ok(value_str) = value.to_str() {
                output.push_str(&format!("  {}: {}\n", name, value_str));
            }
        }

        output
    }
}

/// Get status text for HTTP status code
fn status_text(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        206 => "Partial Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Unknown",
    }
}
