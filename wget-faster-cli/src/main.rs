mod args;
mod output;

use args::Args;
use output::WgetOutput;
use clap::Parser;
use wget_faster_lib::{Downloader, DownloadConfig, ProgressInfo};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use url::Url;
use percent_encoding;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Handle version flag first (before URL validation)
    if args.version {
        print_version();
        std::process::exit(0);
    }

    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("wgetf: {}", e);
        std::process::exit(1);
    }

    // Collect URLs from args and input file
    let mut urls = args.urls.clone();

    // Read URLs from input file if specified
    if let Some(ref input_file) = args.input_file {
        let resolved_input_file = resolve_file_path(input_file);
        match read_urls_from_file(&resolved_input_file, args.force_html, args.base.as_deref()).await {
            Ok(file_urls) => urls.extend(file_urls),
            Err(e) => {
                eprintln!("wgetf: failed to read input file: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Check if no URLs provided
    if urls.is_empty() {
        eprintln!("wgetf: missing URL");
        eprintln!("Usage: wgetf [OPTION]... [URL]...");
        eprintln!();
        eprintln!("Try `wgetf --help' for more options.");
        std::process::exit(1);
    }

    // Build configuration from args
    let config = match build_config(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("wgetf: {}", e);
            std::process::exit(1);
        }
    };

    // Extract values before moving config
    let wait_time = config.wait_time;
    let random_wait = config.random_wait;
    let quota = config.quota;

    // Create downloader
    let downloader = match Downloader::new(config) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("wgetf: failed to create downloader: {}", e);
            std::process::exit(1);
        }
    };

    // Download all URLs
    let mut exit_code = 0;
    let mut total_downloaded: u64 = 0;

    for (i, url) in urls.iter().enumerate() {
        // Check quota before download
        if let Some(q) = quota {
            if total_downloaded >= q {
                eprintln!("wgetf: quota of {} bytes exceeded", q);
                break;
            }
        }

        // Wait between downloads (except for first)
        if i > 0 {
            if let Some(wt) = wait_time {
                let actual_wait = if random_wait {
                    // Random wait between 0.5x and 1.5x wait_time
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let multiplier = rng.gen_range(0.5..=1.5);
                    Duration::from_secs_f64(wt.as_secs_f64() * multiplier)
                } else {
                    wt
                };
                tokio::time::sleep(actual_wait).await;
            }
        }

        match download_url(&downloader, url, &args).await {
            Ok(bytes) => {
                total_downloaded += bytes;
            }
            Err(e) => {
                eprintln!("wgetf: {}", e);

                // Get exit code from error - check if it's a library error first
                if let Some(lib_err) = e.downcast_ref::<wget_faster_lib::Error>() {
                    // Use wget-compatible exit code from library error
                    exit_code = lib_err.exit_code();
                } else {
                    // For other errors, use generic exit code 1
                    exit_code = 1;
                }
            }
        }
    }

    std::process::exit(exit_code);
}

async fn download_url(
    downloader: &Downloader,
    url: &str,
    args: &Args,
) -> Result<u64, Box<dyn std::error::Error>> {
    // Parse URL
    let parsed_url = Url::parse(url)?;

    // Get metadata first if content_disposition is enabled
    let metadata = if args.content_disposition {
        Some(downloader.get_client().get_metadata(url).await?)
    } else {
        None
    };

    // Determine output file name
    let output_path = determine_output_path(&parsed_url, args, metadata.as_ref())?;

    // Create output formatter
    let mut output = WgetOutput::new(
        args.quiet,
        args.verbose || args.debug > 0,
        args.show_progress || (!args.quiet && !args.no_verbose),
    );

    // Print connection info
    let host = parsed_url.host_str().unwrap_or("unknown");
    let port = parsed_url.port().unwrap_or(match parsed_url.scheme() {
        "https" => 443,
        "http" => 80,
        _ => 80,
    });
    output.print_connecting(url, host, port);
    output.print_http_request();

    // Spider mode - just check if exists
    if args.spider {
        // Send HEAD request to check if resource exists
        match downloader.download_to_memory(url).await {
            Ok(_) => {
                output.print_spider_result(url, 200, true);
                return Ok(0);
            }
            Err(e) => {
                // In spider mode, HTTP errors should return exit code 8
                // Extract status code from error
                let status_code = if let Some(pos) = e.to_string().find("Invalid status: ") {
                    e.to_string()[pos + 16..].split_whitespace().next()
                        .and_then(|s| s.parse::<u16>().ok())
                        .unwrap_or(0)
                } else {
                    0
                };

                if status_code >= 400 {
                    // HTTP 4xx/5xx errors
                    output.print_spider_result(url, status_code, false);
                    // Return an error with exit code 8
                    return Err(Box::new(wget_faster_lib::Error::InvalidStatus(status_code)));
                } else if status_code > 0 {
                    // Other status codes (1xx, 2xx, 3xx)
                    output.print_spider_result(url, status_code, true);
                    return Ok(0);
                }

                // Non-HTTP errors
                output.print_error(&format!("spider check failed: {}", e));
                return Err(e.into());
            }
        }
    }

    // Start download
    let start_time = Instant::now();

    // Print saving to file
    if let Some(ref path) = output_path {
        output.print_saving_to(&path.display().to_string());
    }

    // Create progress callback
    let output_for_progress = Arc::new(tokio::sync::Mutex::new(output));
    let output_clone = output_for_progress.clone();

    let progress_callback = Arc::new(move |progress: ProgressInfo| {
        if let Ok(mut out) = output_clone.try_lock() {
            out.update_progress(&progress);
        }
    });

    // Download
    let result = if let Some(path) = output_path {
        // Initialize progress bar
        {
            let mut out = output_for_progress.lock().await;
            out.init_progress(None); // Will be updated on first progress callback
        }

        downloader
            .download_to_file_with_progress(url, path.clone(), Some(progress_callback))
            .await
    } else {
        // Download to stdout
        let bytes = downloader
            .download_to_memory_with_progress(url, Some(progress_callback))
            .await?;

        // Write to stdout
        use std::io::Write;
        std::io::stdout().write_all(&bytes)?;

        return Ok(bytes.len() as u64);
    };

    // Finish progress
    {
        let mut out = output_for_progress.lock().await;
        out.finish_progress();
    }

    match result {
        Ok(download_result) => {
            let elapsed = start_time.elapsed();
            let mut out = output_for_progress.lock().await;

            // Print HTTP response
            out.print_http_response(200, "OK");

            // Print content info
            out.print_content_info(
                download_result.metadata.content_length,
                download_result.metadata.content_type.as_deref(),
            );

            // Print completion message
            let filename = download_result
                .data
                .file_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            out.print_complete(&filename, download_result.data.total_bytes, elapsed);

            Ok(download_result.data.total_bytes)
        }
        Err(e) => {
            let mut out = output_for_progress.lock().await;
            out.print_error(&format!("download failed: {}", e));
            Err(e.into())
        }
    }
}

fn determine_output_path(
    url: &Url,
    args: &Args,
    metadata: Option<&wget_faster_lib::ResourceMetadata>,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    // If -O is specified
    if let Some(ref output_doc) = args.output_document {
        // Special case: -O - means stdout
        if output_doc.to_str() == Some("-") {
            return Ok(None);
        }
        return Ok(Some(output_doc.clone()));
    }

    // Try to extract filename from Content-Disposition if enabled
    let filename = if args.content_disposition {
        metadata
            .and_then(|m| m.content_disposition.as_ref())
            .and_then(|cd| extract_filename_from_content_disposition(cd))
            .or_else(|| {
                // Fall back to URL if Content-Disposition not available
                url.path_segments()
                    .and_then(|segments| segments.last())
                    .filter(|name| !name.is_empty())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "index.html".to_string())
    } else {
        // Extract filename from URL
        url.path_segments()
            .and_then(|segments| segments.last())
            .filter(|name| !name.is_empty())
            .unwrap_or("index.html")
            .to_string()
    };

    let mut path = PathBuf::new();

    // Add directory prefix if specified (unless -n/--no-directories is set)
    if !args.no_directories {
        if let Some(ref prefix) = args.directory_prefix {
            path.push(prefix);
        }
    } else if let Some(ref prefix) = args.directory_prefix {
        // With -n, still use directory_prefix if specified explicitly
        path.push(prefix);
    }

    // Add filename
    path.push(filename);

    // Handle no-clobber
    if args.no_clobber && path.exists() {
        return Err(format!("File '{}' already exists.", path.display()).into());
    }

    Ok(Some(path))
}

fn extract_filename_from_content_disposition(header: &str) -> Option<String> {
    // Parse Content-Disposition header
    // Format: attachment; filename="filename.jpg"
    // or: attachment; filename*=UTF-8''filename.jpg

    for part in header.split(';') {
        let part = part.trim();

        // Handle filename*= (RFC 5987)
        if part.to_lowercase().starts_with("filename*=") {
            if let Some(value) = part.split('=').nth(1) {
                // Remove encoding prefix if present (e.g., "UTF-8''")
                let value = if let Some(pos) = value.find("''") {
                    &value[pos + 2..]
                } else {
                    value
                };
                // URL decode and remove quotes
                let decoded = percent_encoding::percent_decode_str(value)
                    .decode_utf8()
                    .ok()?;
                return Some(decoded.trim_matches('"').to_string());
            }
        }

        // Handle filename= (RFC 2183)
        if part.to_lowercase().starts_with("filename=") {
            if let Some(value) = part.split('=').nth(1) {
                return Some(value.trim_matches('"').to_string());
            }
        }
    }

    None
}

fn build_config(args: &Args) -> Result<DownloadConfig, Box<dyn std::error::Error>> {
    let mut config = DownloadConfig::default();

    // Set timeouts
    if let Some(timeout) = args.timeout {
        config.timeout = Duration::from_secs(timeout);
    }
    if let Some(timeout) = args.connect_timeout {
        config.connect_timeout = Duration::from_secs(timeout);
    }
    if let Some(timeout) = args.read_timeout {
        config.read_timeout = Duration::from_secs(timeout);
    }

    // Set retry configuration
    config.retry.max_retries = args.tries;
    if args.retry_connrefused {
        config.retry.retry_on_conn_refused = true;
    }

    // Set user agent
    if let Some(ref ua) = args.user_agent {
        config.user_agent = ua.clone();
    }

    // Set custom headers
    for header in &args.header {
        if let Some((key, value)) = header.split_once(':') {
            config.headers.insert(
                key.trim().to_string(),
                value.trim().to_string(),
            );
        }
    }

    // Set cookies
    config.enable_cookies = !args.no_cookies;
    if let Some(ref cookie_file) = args.load_cookies {
        config.cookie_file = Some(cookie_file.clone());
    }

    // Set SSL verification
    config.verify_ssl = !args.no_check_certificate;

    // Set certificates
    if let Some(ref cert) = args.ca_certificate {
        config.ca_cert = Some(cert.clone());
    }
    if let Some(ref cert) = args.certificate {
        config.client_cert = Some(cert.clone());
    }

    // Set redirect following
    config.follow_redirects = true;
    if let Some(max_redir) = args.max_redirect {
        config.max_redirects = max_redir;
    }

    // Set speed limit
    if let Some(ref rate) = args.limit_rate {
        config.speed_limit = parse_rate(rate)?;
    }

    // Set authentication
    if let Some(ref user) = args.http_user {
        let password = args.http_password.clone().unwrap_or_default();
        config.auth = Some(wget_faster_lib::AuthConfig {
            username: user.clone(),
            password,
            auth_type: wget_faster_lib::AuthType::Basic,
        });
    }

    // Set authentication without challenge
    config.auth_no_challenge = args.auth_no_challenge;

    // Set HTTP method
    if let Some(ref method) = args.method {
        config.method = wget_faster_lib::HttpMethod::from_str(method)
            .ok_or_else(|| format!("Invalid HTTP method: {}", method))?;
    }

    // Set POST data
    if let Some(ref post_data) = args.post_data {
        config.method = wget_faster_lib::HttpMethod::Post;
        config.body_data = Some(post_data.as_bytes().to_vec());
    } else if let Some(ref post_file) = args.post_file {
        config.method = wget_faster_lib::HttpMethod::Post;
        let resolved_post_file = resolve_file_path(post_file);
        let data = std::fs::read(&resolved_post_file)
            .map_err(|e| format!("Failed to read POST file '{}': {}", resolved_post_file.display(), e))?;
        config.body_data = Some(data);
    }

    // Set body data (for --body-data and --body-file)
    if let Some(ref body_data) = args.body_data {
        config.body_data = Some(body_data.as_bytes().to_vec());
    } else if let Some(ref body_file) = args.body_file {
        let resolved_body_file = resolve_file_path(body_file);
        let data = std::fs::read(&resolved_body_file)
            .map_err(|e| format!("Failed to read body file '{}': {}", resolved_body_file.display(), e))?;
        config.body_data = Some(data);
    }

    // Set referer
    if let Some(ref referer) = args.referer {
        config.referer = Some(referer.clone());
    }

    // Set proxy
    if let Some(ref user) = args.proxy_user {
        let password = args.proxy_password.clone().unwrap_or_default();
        if let Some(proxy_url) = std::env::var("http_proxy").ok()
            .or_else(|| std::env::var("https_proxy").ok())
            .or_else(|| std::env::var("HTTP_PROXY").ok())
            .or_else(|| std::env::var("HTTPS_PROXY").ok())
        {
            config.proxy = Some(wget_faster_lib::ProxyConfig {
                url: proxy_url,
                auth: Some((user.clone(), password)),
            });
        }
    }

    // Set compression
    config.enable_compression = !matches!(args.compression.as_deref(), Some("none"));

    // Set HTTP keep-alive
    config.http_keep_alive = !args.no_http_keep_alive;

    // Set wait time
    if let Some(wait) = args.wait {
        config.wait_time = Some(Duration::from_secs(wait));
    }

    // Set random wait
    config.random_wait = args.random_wait;

    // Set wait retry
    if let Some(waitretry) = args.waitretry {
        config.wait_retry = Some(Duration::from_secs(waitretry));
    }

    // Set quota
    if let Some(ref quota_str) = args.quota {
        config.quota = parse_quota(quota_str)?;
    }

    // Set timestamping
    config.timestamping = args.timestamping;
    config.if_modified_since = !args.no_if_modified_since;
    config.use_server_timestamps = !args.no_use_server_timestamps;

    // Set content disposition
    config.content_disposition = args.content_disposition;

    // Set save headers
    config.save_headers = args.save_headers;

    // Set content on error
    config.content_on_error = args.content_on_error;

    // Set verbose mode
    config.verbose = args.verbose || args.debug > 0;

    Ok(config)
}

fn parse_quota(quota: &str) -> Result<Option<u64>, Box<dyn std::error::Error>> {
    let quota = quota.trim().to_lowercase();

    // Parse quota with suffix (k, m, g)
    let (num_str, multiplier) = if quota.ends_with('k') {
        (&quota[..quota.len() - 1], 1024)
    } else if quota.ends_with('m') {
        (&quota[..quota.len() - 1], 1024 * 1024)
    } else if quota.ends_with('g') {
        (&quota[..quota.len() - 1], 1024 * 1024 * 1024)
    } else {
        (quota.as_str(), 1)
    };

    let num: f64 = num_str.parse()?;
    let bytes = (num * multiplier as f64) as u64;

    Ok(Some(bytes))
}

fn parse_rate(rate: &str) -> Result<Option<u64>, Box<dyn std::error::Error>> {
    let rate = rate.trim().to_lowercase();

    // Parse rate with suffix (k, m, g)
    let (num_str, multiplier) = if rate.ends_with('k') {
        (&rate[..rate.len() - 1], 1024)
    } else if rate.ends_with('m') {
        (&rate[..rate.len() - 1], 1024 * 1024)
    } else if rate.ends_with('g') {
        (&rate[..rate.len() - 1], 1024 * 1024 * 1024)
    } else {
        (rate.as_str(), 1)
    };

    let num: f64 = num_str.parse()?;
    let bytes_per_sec = (num * multiplier as f64) as u64;

    Ok(Some(bytes_per_sec))
}

async fn read_urls_from_file(
    path: &PathBuf,
    force_html: bool,
    base_url: Option<&str>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use tokio::fs::File;
    use tokio::io::{AsyncBufReadExt, BufReader};

    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut urls = Vec::new();

    if force_html {
        // Parse HTML and extract links
        let content = tokio::fs::read_to_string(path).await?;
        urls.extend(extract_urls_from_html(&content, base_url)?);
    } else {
        // Read URLs line by line
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Resolve relative URLs if base is provided
            let url = if let Some(base) = base_url {
                resolve_url(base, line)?
            } else {
                line.to_string()
            };

            urls.push(url);
        }
    }

    Ok(urls)
}

fn extract_urls_from_html(html: &str, base_url: Option<&str>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);

    // Selectors for different link types
    let a_selector = Selector::parse("a[href]").unwrap();
    let img_selector = Selector::parse("img[src]").unwrap();
    let link_selector = Selector::parse("link[href]").unwrap();
    let script_selector = Selector::parse("script[src]").unwrap();

    let mut urls = Vec::new();

    // Extract URLs from <a> tags
    for element in document.select(&a_selector) {
        if let Some(href) = element.value().attr("href") {
            let url = if let Some(base) = base_url {
                resolve_url(base, href)?
            } else {
                href.to_string()
            };
            urls.push(url);
        }
    }

    // Extract URLs from <img> tags
    for element in document.select(&img_selector) {
        if let Some(src) = element.value().attr("src") {
            let url = if let Some(base) = base_url {
                resolve_url(base, src)?
            } else {
                src.to_string()
            };
            urls.push(url);
        }
    }

    // Extract URLs from <link> tags
    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            let url = if let Some(base) = base_url {
                resolve_url(base, href)?
            } else {
                href.to_string()
            };
            urls.push(url);
        }
    }

    // Extract URLs from <script> tags
    for element in document.select(&script_selector) {
        if let Some(src) = element.value().attr("src") {
            let url = if let Some(base) = base_url {
                resolve_url(base, src)?
            } else {
                src.to_string()
            };
            urls.push(url);
        }
    }

    Ok(urls)
}

fn resolve_url(base: &str, relative: &str) -> Result<String, Box<dyn std::error::Error>> {
    let base_url = Url::parse(base)?;
    let resolved = base_url.join(relative)?;
    Ok(resolved.to_string())
}

fn resolve_file_path(path: &PathBuf) -> PathBuf {
    // If path is absolute, return as-is
    if path.is_absolute() {
        return path.clone();
    }

    // Otherwise, resolve relative to current working directory
    match std::env::current_dir() {
        Ok(cwd) => cwd.join(path),
        Err(_) => path.clone(), // Fallback to original path if CWD unavailable
    }
}

fn print_version() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    println!("GNU Wget {} built on {}.", VERSION, std::env::consts::OS);
    println!();
    println!("+digest +https +ipv6 -iri +large-file +nls +ssl/rustls");
    println!();
    println!("Features:");
    println!("    +http2         HTTP/2 support via reqwest");
    println!("    +parallel      Parallel chunk downloads");
    println!("    +adaptive      Adaptive performance tuning");
    println!("    +cookies       Cookie support (Netscape format)");
    println!("    +compression   gzip, deflate, brotli");
    println!("    +recursive     Recursive downloads with HTML parsing");
    println!("    +timestamping  If-Modified-Since support");
    println!("    +resume        Resume partial downloads");
    println!();
    println!("Copyright (C) 2024 wget-faster contributors");
    println!("License BSD-3-Clause: BSD 3-Clause License");
    println!("This is free software: you are free to change and redistribute it.");
    println!("There is NO WARRANTY, to the extent permitted by law.");
}
