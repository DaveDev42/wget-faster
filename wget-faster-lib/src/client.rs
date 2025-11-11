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
        if let Some(proxy_config) = &config.proxy {
            let mut proxy = reqwest::Proxy::all(&proxy_config.url)
                .map_err(|e| Error::ConfigError(format!("Invalid proxy URL: {}", e)))?;

            if let Some((username, password)) = &proxy_config.auth {
                proxy = proxy.basic_auth(username, password);
            }

            builder = builder.proxy(proxy);
        }

        // Configure authentication
        if let Some(_auth_config) = &config.auth {
            // Basic auth will be added per-request
            builder = builder;
        }

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
    pub async fn get_metadata(&self, url: &str) -> Result<ResourceMetadata> {
        let response = self.client.head(url).send().await?;

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

        Ok(ResourceMetadata {
            supports_range,
            content_length,
            last_modified,
            etag,
            content_type,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ResourceMetadata {
    pub supports_range: bool,
    pub content_length: Option<u64>,
    pub last_modified: Option<String>,
    pub etag: Option<String>,
    pub content_type: Option<String>,
}
