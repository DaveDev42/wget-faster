mod args;
mod output;

use anyhow::{anyhow, Context, Result};
use args::Args;
use clap::Parser;
use output::WgetOutput;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use url::Url;
use wget_faster_lib::{DownloadConfig, Downloader, ProgressInfo};

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for structured logging
    // Use RUST_LOG environment variable to control log level
    // Example: RUST_LOG=debug wgetf https://example.com
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_target(false) // Don't print module paths
        .with_thread_ids(false) // Don't print thread IDs
        .with_line_number(true) // Show line numbers for debugging
        .init();

    // Pre-process arguments to handle wget-style multi-character short flags
    // GNU wget supports flags like -nH (no-host-directories) and -np (no-parent)
    // which clap doesn't support natively
    let preprocessed_args = preprocess_args(std::env::args().collect());
    let mut args = Args::parse_from(preprocessed_args);

    // Handle version flag first (before URL validation)
    if args.version {
        print_version();
        std::process::exit(0);
    }

    // Process -e/--execute commands
    if let Some(execute_cmd) = args.execute.clone() {
        if let Err(e) = process_execute_command(&mut args, &execute_cmd) {
            eprintln!("wgetf: {e}");
            std::process::exit(1);
        }
    }

    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("wgetf: {e}");
        std::process::exit(1);
    }

    // Collect URLs from args and input file
    let mut urls = args.urls.clone();

    // Read URLs from input file if specified
    if let Some(ref input_file) = args.input_file {
        // Check if input_file is a URL or a local file path
        let input_str = input_file.to_str().unwrap_or("");

        if input_str.starts_with("http://")
            || input_str.starts_with("https://")
            || input_str.starts_with("ftp://")
        {
            // Input file is a URL - download it first
            match download_input_file_from_url(input_str, args.force_html, args.base.as_deref())
                .await
            {
                Ok(file_urls) => urls.extend(file_urls),
                Err(e) => {
                    eprintln!("wgetf: failed to read input file from URL: {e}");
                    std::process::exit(1);
                },
            }
        } else {
            // Input file is a local file path
            let resolved_input_file = resolve_file_path(input_file);
            match read_urls_from_file(&resolved_input_file, args.force_html, args.base.as_deref())
                .await
            {
                Ok(file_urls) => urls.extend(file_urls),
                Err(e) => {
                    eprintln!("wgetf: failed to read input file: {e}");
                    std::process::exit(1);
                },
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
            eprintln!("wgetf: {e}");
            std::process::exit(1);
        },
    };

    // Extract values before moving config
    let wait_time = config.wait_time;
    let random_wait = config.random_wait;
    let quota = config.quota;

    // Check if recursive mode is enabled
    if args.recursive {
        // Recursive download mode
        let recursive_config = build_recursive_config(&args);
        let mut recursive_downloader =
            match wget_faster_lib::RecursiveDownloader::new(config, recursive_config) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("wgetf: failed to create recursive downloader: {e}");
                    std::process::exit(1);
                },
            };

        let mut exit_code = 0;

        // Process each URL recursively
        for url in &urls {
            // Determine output directory
            let output_dir = if let Some(ref prefix) = args.directory_prefix {
                PathBuf::from(prefix)
            } else {
                PathBuf::from(".")
            };

            match recursive_downloader
                .download_recursive(url, &output_dir)
                .await
            {
                Ok(_files) => {
                    // Check if there were broken links in spider mode
                    if args.spider {
                        let broken_links = recursive_downloader.broken_links();
                        if !broken_links.is_empty() {
                            exit_code = 8; // wget exit code for broken links
                        }
                    }
                },
                Err(e) => {
                    eprintln!("wgetf: recursive download failed: {e}");
                    exit_code = 1;
                },
            }
        }

        std::process::exit(exit_code);
    }

    // Create downloader for non-recursive mode
    let downloader = match Downloader::new(config) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("wgetf: failed to create downloader: {e}");
            std::process::exit(1);
        },
    };

    // Download all URLs (non-recursive mode)
    let mut exit_code = 0;
    let mut total_downloaded: u64 = 0;

    for (i, url) in urls.iter().enumerate() {
        // Check quota before download
        if let Some(q) = quota {
            if total_downloaded >= q {
                eprintln!("wgetf: quota of {q} bytes exceeded");
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

        // Retry loop for 5xx errors and other transient failures
        let mut attempt = 0;
        let max_tries = downloader.get_client().config().retry.max_retries;

        loop {
            attempt += 1;

            match download_url(&downloader, url, &args).await {
                Ok(bytes) => {
                    total_downloaded += bytes;
                    break;
                },
                Err(e) => {
                    // Check if error is retryable
                    let should_retry =
                        if let Some(lib_err) = e.downcast_ref::<wget_faster_lib::Error>() {
                            // Check if this is a retryable status code
                            if let wget_faster_lib::Error::InvalidStatus(status) = lib_err {
                                downloader
                                    .get_client()
                                    .config()
                                    .retry
                                    .retry_on_status
                                    .contains(status)
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                    if should_retry && attempt < max_tries {
                        // Calculate backoff delay
                        let retry_config = &downloader.get_client().config().retry;
                        let delay = retry_config.initial_delay.as_secs_f64()
                            * retry_config.backoff_multiplier.powi((attempt - 1) as i32);
                        let delay = Duration::from_secs_f64(
                            delay.min(retry_config.max_delay.as_secs_f64()),
                        );

                        eprintln!(
                            "wgetf: retrying in {} seconds... (attempt {}/{})",
                            delay.as_secs(),
                            attempt,
                            max_tries
                        );

                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    // Not retryable or max retries reached
                    eprintln!("wgetf: {e}");

                    // Get exit code from error - check if it's a library error first
                    if let Some(lib_err) = e.downcast_ref::<wget_faster_lib::Error>() {
                        // Use wget-compatible exit code from library error
                        exit_code = lib_err.exit_code();
                    } else {
                        // For other errors, use generic exit code 1
                        exit_code = 1;
                    }
                    break;
                },
            }
        }
    }

    std::process::exit(exit_code);
}

async fn download_url(downloader: &Downloader, url: &str, args: &Args) -> Result<u64> {
    // Parse URL
    let parsed_url = Url::parse(url).with_context(|| format!("Failed to parse URL: {url}"))?;

    // Get metadata first if content_disposition is enabled
    let metadata = if args.content_disposition {
        Some(
            downloader
                .get_client()
                .get_metadata(url)
                .await
                .with_context(|| format!("Failed to get metadata from: {url}"))?,
        )
    } else {
        None
    };

    // Determine output file name
    let output_path = determine_output_path(&parsed_url, args, metadata.as_ref())
        .with_context(|| "Failed to determine output file path")?;

    // Create output formatter
    let output = if let Some(ref log_file) = args.output_file {
        // Use -o (truncate mode)
        match WgetOutput::with_log_file(
            args.quiet,
            args.verbose || args.debug > 0,
            args.show_progress || (!args.quiet && !args.no_verbose),
            log_file.clone(),
            false,
        ) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("wgetf: failed to open log file '{}': {}", log_file.display(), e);
                std::process::exit(3); // File I/O error
            },
        }
    } else if let Some(ref log_file) = args.append_output {
        // Use -a (append mode)
        match WgetOutput::with_log_file(
            args.quiet,
            args.verbose || args.debug > 0,
            args.show_progress || (!args.quiet && !args.no_verbose),
            log_file.clone(),
            true,
        ) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("wgetf: failed to open log file '{}': {}", log_file.display(), e);
                std::process::exit(3); // File I/O error
            },
        }
    } else {
        // Default to terminal output
        WgetOutput::new(
            args.quiet,
            args.verbose || args.debug > 0,
            args.show_progress || (!args.quiet && !args.no_verbose),
        )
    };

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
            },
            Err(e) => {
                // In spider mode, HTTP errors should return exit code 8
                // Extract status code from error
                let status_code = if let Some(pos) = e.to_string().find("Invalid status: ") {
                    e.to_string()[pos + 16..]
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<u16>().ok())
                        .unwrap_or(0)
                } else {
                    0
                };

                if status_code >= 400 {
                    // HTTP 4xx/5xx errors
                    output.print_spider_result(url, status_code, false);
                    // Return an error with exit code 8
                    return Err(wget_faster_lib::Error::InvalidStatus(status_code).into());
                } else if status_code > 0 {
                    // Other status codes (1xx, 2xx, 3xx)
                    output.print_spider_result(url, status_code, true);
                    return Ok(0);
                }

                // Non-HTTP errors
                output.print_error(&format!("spider check failed: {e}"));
                return Err(e.into());
            },
        }
    }

    // Start download
    let start_time = Instant::now();

    // Print saving to file
    if let Some(ref path) = output_path {
        output.print_saving_to(&path.display().to_string());

        // When using -O (output-document), create the file before download
        // This matches GNU wget behavior: the file is created even if download fails
        // Only do this for -O flag, not for other output modes
        if args.output_document.is_some() {
            // Create empty file (or truncate if exists)
            if let Err(e) = std::fs::File::create(path) {
                eprintln!("wgetf: cannot write to '{}': {}", path.display(), e);
                std::process::exit(3); // File I/O error
            }
        }
    }

    // Create progress callback
    let output_for_progress = Arc::new(tokio::sync::Mutex::new(output));
    let output_clone = output_for_progress.clone();

    let progress_callback = Arc::new(move |progress: ProgressInfo| {
        if let Ok(out) = output_clone.try_lock() {
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
            .await
            .with_context(|| format!("Failed to download: {url}"))?;

        // Write to stdout
        use std::io::Write;
        std::io::stdout()
            .write_all(&bytes)
            .context("Failed to write to stdout")?;

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
            let out = output_for_progress.lock().await;

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
                .map_or_else(|| "unknown".to_string(), |p| p.display().to_string());

            out.print_complete(&filename, download_result.data.total_bytes, elapsed);

            Ok(download_result.data.total_bytes)
        },
        Err(e) => {
            let out = output_for_progress.lock().await;
            out.print_error(&format!("download failed: {e}"));
            Err(e.into())
        },
    }
}

fn determine_output_path(
    url: &Url,
    args: &Args,
    metadata: Option<&wget_faster_lib::ResourceMetadata>,
) -> Result<Option<PathBuf>> {
    // If -O is specified
    if let Some(ref output_doc) = args.output_document {
        // Special case: -O - means stdout
        if output_doc.to_str() == Some("-") {
            return Ok(None);
        }
        return Ok(Some(output_doc.clone()));
    }

    // Try to extract filename from Content-Disposition if enabled
    let mut filename = if args.content_disposition {
        metadata
            .and_then(|m| m.content_disposition.as_ref())
            .and_then(|cd| extract_filename_from_content_disposition(cd))
            .or_else(|| {
                // Fall back to URL if Content-Disposition not available
                url.path_segments()
                    .and_then(|mut segments| segments.next_back())
                    .filter(|name| !name.is_empty())
                    .map(std::string::ToString::to_string)
            })
            .unwrap_or_else(|| "index.html".to_string())
    } else {
        // Extract filename from URL
        url.path_segments()
            .and_then(|mut segments| segments.next_back())
            .filter(|name| !name.is_empty())
            .unwrap_or("index.html")
            .to_string()
    };

    // Apply filename restrictions if specified
    if let Some(ref restrict_str) = args.restrict_file_names {
        // Parse and apply restrictions
        let mut restrictions = Vec::new();
        for mode in restrict_str.split(',') {
            let mode = mode.trim();
            if let Ok(restriction) = mode.parse::<wget_faster_lib::FilenameRestriction>() {
                restrictions.push(restriction);
            }
        }

        // Apply all restrictions to filename
        filename = wget_faster_lib::apply_filename_restrictions(&filename, &restrictions);
    }

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
    path.push(&filename);

    // Handle no-clobber
    if args.no_clobber && path.exists() {
        return Err(anyhow!("File '{}' already exists.", path.display()));
    }

    // Handle duplicate filenames by adding .1, .2, .3 suffix
    // This matches wget behavior for Content-Disposition filenames
    // Skip this for timestamping (-N) or continue (-c) mode where we want to use the same file
    // EXCEPT: If --start-pos is used with --continue, we still create a numbered file
    let should_number_file = path.exists()
        && !args.no_clobber
        && !args.timestamping
        && (!args.continue_download || args.start_pos.is_some());

    if should_number_file {
        let mut counter = 1;
        loop {
            let mut new_path = PathBuf::new();

            // Add directory prefix
            if !args.no_directories {
                if let Some(ref prefix) = args.directory_prefix {
                    new_path.push(prefix);
                }
            } else if let Some(ref prefix) = args.directory_prefix {
                new_path.push(prefix);
            }

            // Add filename with counter suffix
            new_path.push(format!("{filename}.{counter}"));

            if !new_path.exists() {
                path = new_path;
                break;
            }

            counter += 1;

            // Safety check: prevent infinite loop
            if counter > 9999 {
                return Err(anyhow!("Too many duplicate files"));
            }
        }
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

fn process_execute_command(args: &mut Args, command: &str) -> Result<(), String> {
    // Parse execute command in the format "key=value"
    // Currently supports: contentdisposition=on/off

    let command = command.trim();

    if let Some((key, value)) = command.split_once('=') {
        let key = key.trim().to_lowercase();
        let value = value.trim().to_lowercase();

        match key.as_str() {
            "contentdisposition" => match value.as_str() {
                "on" | "1" | "true" => {
                    args.content_disposition = true;
                    Ok(())
                },
                "off" | "0" | "false" => {
                    args.content_disposition = false;
                    Ok(())
                },
                _ => Err(format!("Invalid value for contentdisposition: {value}")),
            },
            _ => {
                // For unknown commands, silently ignore (wget behavior)
                Ok(())
            },
        }
    } else {
        Err(format!("Invalid execute command format: {command}"))
    }
}

fn build_config(args: &Args) -> Result<DownloadConfig> {
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
            config
                .headers
                .insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    // Set cookies
    config.enable_cookies = !args.no_cookies;
    if let Some(ref cookie_file) = args.load_cookies {
        config.cookie_file = Some(resolve_file_path(cookie_file));
    }

    // Set SSL verification
    config.verify_ssl = !args.no_check_certificate;

    // Set certificates
    if let Some(ref cert) = args.ca_certificate {
        config.ca_cert = Some(resolve_file_path(cert));
    }
    if let Some(ref cert) = args.certificate {
        config.client_cert = Some(resolve_file_path(cert));
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

    // Also check generic --user flag (fallback to --http-user)
    if config.auth.is_none() {
        if let Some(ref user) = args.user {
            let password = args.password.clone().unwrap_or_default();
            config.auth = Some(wget_faster_lib::AuthConfig {
                username: user.clone(),
                password,
                auth_type: wget_faster_lib::AuthType::Basic,
            });
        }
    }

    // Set authentication without challenge (preemptive auth)
    // Only enable preemptive auth when --auth-no-challenge is explicitly provided
    // Default behavior: wait for 401/407 challenge before sending credentials
    config.auth_no_challenge = args.auth_no_challenge;

    // Set HTTP method
    if let Some(ref method) = args.method {
        config.method = method
            .parse::<wget_faster_lib::HttpMethod>()
            .map_err(|e| anyhow!("{e}"))?;
    }

    // Set POST data
    if let Some(ref post_data) = args.post_data {
        config.method = wget_faster_lib::HttpMethod::Post;
        config.body_data = Some(post_data.as_bytes().to_vec());
    } else if let Some(ref post_file) = args.post_file {
        config.method = wget_faster_lib::HttpMethod::Post;
        let resolved_post_file = resolve_file_path(post_file);
        let data = match std::fs::read(&resolved_post_file) {
            Ok(d) => d,
            Err(e) => {
                // File I/O error - exit with code 3
                eprintln!(
                    "wgetf: Failed to read POST file '{}': {}",
                    resolved_post_file.display(),
                    e
                );
                std::process::exit(3);
            },
        };
        config.body_data = Some(data);
    }

    // Set body data (for --body-data and --body-file)
    if let Some(ref body_data) = args.body_data {
        config.body_data = Some(body_data.as_bytes().to_vec());
    } else if let Some(ref body_file) = args.body_file {
        let resolved_body_file = resolve_file_path(body_file);
        let data = match std::fs::read(&resolved_body_file) {
            Ok(d) => d,
            Err(e) => {
                // File I/O error - exit with code 3
                eprintln!(
                    "wgetf: Failed to read body file '{}': {}",
                    resolved_body_file.display(),
                    e
                );
                std::process::exit(3);
            },
        };
        config.body_data = Some(data);
    }

    // Set referer
    if let Some(ref referer) = args.referer {
        config.referer = Some(referer.clone());
    }

    // Set proxy configuration
    // Check for proxy URL from environment variables (unless --no-proxy is set)
    if !args.no_proxy {
        if let Some(proxy_url) = std::env::var("http_proxy")
            .ok()
            .or_else(|| std::env::var("https_proxy").ok())
            .or_else(|| std::env::var("HTTP_PROXY").ok())
            .or_else(|| std::env::var("HTTPS_PROXY").ok())
        {
            // Parse no_proxy environment variable
            let no_proxy_list = std::env::var("no_proxy")
                .or_else(|_| std::env::var("NO_PROXY"))
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();

            // Set up proxy authentication if provided
            let auth = if let Some(ref user) = args.proxy_user {
                let password = args.proxy_password.clone().unwrap_or_default();
                Some((user.clone(), password))
            } else {
                None
            };

            config.proxy = Some(wget_faster_lib::ProxyConfig {
                url: proxy_url,
                auth,
                no_proxy: no_proxy_list,
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
    // In quiet mode, default to NOT saving error pages (unless explicitly requested)
    // This matches wget behavior: --quiet suppresses error page downloads
    config.content_on_error = if args.quiet && !args.content_on_error {
        false
    } else {
        args.content_on_error
    };

    // Set verbose mode
    config.verbose = args.verbose || args.debug > 0;

    // Set start position
    config.start_pos = args.start_pos;

    // Set HTTPS-only mode
    config.https_only = args.https_only;

    // Disable parallel downloads if --no-parallel is set
    // This makes wget-faster behave exactly like GNU wget (no HEAD requests, sequential only)
    if args.no_parallel {
        config.parallel_chunks = 1; // Disable parallel chunks
        config.parallel_threshold = 0; // Disable threshold check
    }

    // Parse --restrict-file-names
    if let Some(ref restrict_str) = args.restrict_file_names {
        // Parse comma-separated list of restrictions
        for mode in restrict_str.split(',') {
            let mode = mode.trim();
            match mode.parse::<wget_faster_lib::FilenameRestriction>() {
                Ok(restriction) => config.restrict_file_names.push(restriction),
                Err(e) => return Err(anyhow!("{e}")),
            }
        }
    }

    Ok(config)
}

fn parse_quota(quota: &str) -> Result<Option<u64>> {
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
    let bytes = (num * f64::from(multiplier)) as u64;

    Ok(Some(bytes))
}

fn parse_rate(rate: &str) -> Result<Option<u64>> {
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
    let bytes_per_sec = (num * f64::from(multiplier)) as u64;

    Ok(Some(bytes_per_sec))
}

async fn download_input_file_from_url(
    url: &str,
    force_html: bool,
    base_url: Option<&str>,
) -> Result<Vec<String>> {
    // Create a simple downloader to fetch the input file
    let config = DownloadConfig::default();
    let downloader =
        Downloader::new(config).context("Failed to create downloader for input file")?;

    // Determine the output filename from the URL
    let parsed_url =
        Url::parse(url).with_context(|| format!("Failed to parse input file URL: {url}"))?;
    let filename = parsed_url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .filter(|name| !name.is_empty())
        .unwrap_or("index.html");

    let output_path = PathBuf::from(filename);

    // Download the input file to disk (matching wget behavior)
    downloader
        .download_to_file(url, output_path.clone())
        .await
        .with_context(|| format!("Failed to download input file from URL: {url}"))?;

    // Read the downloaded file to extract URLs
    let content = tokio::fs::read_to_string(&output_path)
        .await
        .with_context(|| {
            format!("Failed to read downloaded input file: {}", output_path.display())
        })?;

    let mut urls = Vec::new();

    if force_html {
        // Parse HTML and extract links
        urls.extend(extract_urls_from_html(&content, base_url)?);
    } else {
        // Read URLs line by line
        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Resolve relative URLs if base is provided
            let resolved_url = if let Some(base) = base_url {
                resolve_url(base, line)?
            } else {
                line.to_string()
            };

            urls.push(resolved_url);
        }
    }

    Ok(urls)
}

async fn read_urls_from_file(
    path: &PathBuf,
    force_html: bool,
    base_url: Option<&str>,
) -> Result<Vec<String>> {
    use tokio::fs::File;
    use tokio::io::{AsyncBufReadExt, BufReader};

    let file = File::open(path)
        .await
        .with_context(|| format!("Failed to open input file: {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut urls = Vec::new();

    if force_html {
        // Parse HTML and extract links
        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read HTML from file: {}", path.display()))?;
        urls.extend(extract_urls_from_html(&content, base_url).with_context(|| {
            format!("Failed to extract URLs from HTML file: {}", path.display())
        })?);
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

fn extract_urls_from_html(html: &str, base_url: Option<&str>) -> Result<Vec<String>> {
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

fn resolve_url(base: &str, relative: &str) -> Result<String> {
    let base_url = Url::parse(base).with_context(|| format!("Failed to parse base URL: {base}"))?;
    let resolved = base_url.join(relative).with_context(|| {
        format!("Failed to resolve relative URL '{relative}' against base '{base}'")
    })?;
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

fn build_recursive_config(args: &Args) -> wget_faster_lib::RecursiveConfig {
    let mut config = wget_faster_lib::RecursiveConfig::default();

    // Set recursion depth (0 = infinite, default = 5)
    config.max_depth = if let Some(ref level_str) = args.level {
        level_str.parse().unwrap_or(5)
    } else {
        5
    };

    // Set spider mode
    config.spider = args.spider;

    // Set span_hosts (follow links to other domains)
    config.span_hosts = args.span_hosts;

    // Set page requisites (download CSS, JS, images)
    config.page_requisites = args.page_requisites;

    // Set no_parent (don't ascend to parent directory)
    config.no_parent = args.no_parent;

    // Set no_host_directories (don't create hostname directories)
    config.no_host_directories = args.no_host_directories;

    // Set convert_links (-k flag)
    config.convert_links = args.convert_links;

    // Set backup_converted (-K flag)
    config.backup_converted = args.backup_converted;

    // Set adjust_extension (-E flag)
    config.adjust_extension = args.adjust_extension;

    // Set rejected_log (--rejected-log)
    config.rejected_log = args.rejected_log.clone();

    // Set no_directories (-nd/--no-directories)
    config.no_directories = args.no_directories;

    config
}

/// Pre-process command-line arguments to expand wget-style multi-character short flags
///
/// GNU wget supports multi-character short flags like:
/// - `-nH` for `--no-host-directories`
/// - `-np` for `--no-parent`
///
/// Since clap doesn't support multi-character short flags, we expand them here.
fn preprocess_args(args: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();

    for arg in args {
        // Check if this is a short flag starting with `-` (but not `--`)
        if arg.starts_with('-') && !arg.starts_with("--") && arg.len() > 2 {
            // Handle multi-character short flags
            match arg.as_str() {
                "-nH" => {
                    result.push("--no-host-directories".to_string());
                },
                "-np" => {
                    result.push("--no-parent".to_string());
                },
                "-nv" => {
                    result.push("--no-verbose".to_string());
                },
                _ => {
                    // For other combinations, try to split into individual flags
                    // This handles cases like "-qO" -> "-q -O"
                    result.push(arg);
                },
            }
        } else {
            result.push(arg);
        }
    }

    result
}

fn print_version() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    // Match GNU wget format for test compatibility
    // First line: program name and version
    println!("GNU Wget {} built on {}.", VERSION, std::env::consts::OS);
    println!();
    // Capabilities on next line (no header - tests parse this line directly)
    println!("+digest +https +ipv6 -iri +large-file +nls +ntlm +opie +ssl/rustls");
    println!();
    // Additional features (wget-faster specific)
    println!("wget-faster features:");
    println!("  +http2         HTTP/2 support via reqwest");
    println!("  +parallel      Parallel chunk downloads");
    println!("  +adaptive      Adaptive performance tuning");
    println!("  +cookies       Cookie support (Netscape format)");
    println!("  +compression   gzip, deflate, brotli");
    println!("  +recursive     Recursive downloads with HTML parsing");
    println!("  +timestamping  If-Modified-Since support");
    println!("  +resume        Resume partial downloads");
    println!();
    println!("Copyright (C) 2024 wget-faster contributors");
    println!("License BSD-3-Clause: BSD 3-Clause License");
    println!("This is free software: you are free to change and redistribute it.");
    println!("There is NO WARRANTY, to the extent permitted by law.");
}
