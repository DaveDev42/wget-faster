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

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("wget-faster: {}", e);
        std::process::exit(1);
    }

    // Check if no URLs provided
    if args.urls.is_empty() && args.input_file.is_none() {
        eprintln!("wget-faster: missing URL");
        eprintln!("Usage: wget-faster [OPTION]... [URL]...");
        eprintln!();
        eprintln!("Try `wget-faster --help' for more options.");
        std::process::exit(1);
    }

    // Build configuration from args
    let config = match build_config(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("wget-faster: {}", e);
            std::process::exit(1);
        }
    };

    // Create downloader
    let downloader = match Downloader::new(config) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("wget-faster: failed to create downloader: {}", e);
            std::process::exit(1);
        }
    };

    // Download all URLs
    let mut exit_code = 0;

    for url in &args.urls {
        if let Err(e) = download_url(&downloader, url, &args).await {
            eprintln!("wget-faster: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

async fn download_url(
    downloader: &Downloader,
    url: &str,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse URL
    let parsed_url = Url::parse(url)?;

    // Determine output file name
    let output_path = determine_output_path(&parsed_url, args)?;

    // Create output formatter
    let mut output = WgetOutput::new(
        args.quiet,
        args.verbose || args.debug,
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
        // TODO: Implement spider mode
        output.print_spider_result(url, 200, true);
        return Ok(());
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

        return Ok(());
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

            Ok(())
        }
        Err(e) => {
            let mut out = output_for_progress.lock().await;
            out.print_error(&format!("download failed: {}", e));
            Err(e.into())
        }
    }
}

fn determine_output_path(url: &Url, args: &Args) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    // If -O is specified
    if let Some(ref output_doc) = args.output_document {
        // Special case: -O - means stdout
        if output_doc.to_str() == Some("-") {
            return Ok(None);
        }
        return Ok(Some(output_doc.clone()));
    }

    // Extract filename from URL
    let filename = url
        .path_segments()
        .and_then(|segments| segments.last())
        .filter(|name| !name.is_empty())
        .unwrap_or("index.html");

    let mut path = PathBuf::new();

    // Add directory prefix if specified
    if let Some(ref prefix) = args.directory_prefix {
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

    // Set proxy
    // TODO: Parse from environment or args

    // Set compression
    config.enable_compression = !matches!(args.compression.as_deref(), Some("none"));

    // Set verbose mode
    config.verbose = args.verbose || args.debug;

    Ok(config)
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
