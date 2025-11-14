use indicatif::{ProgressBar, ProgressStyle};
use wget_faster_lib::{ProgressInfo, format_bytes, format_bytes_per_sec};
use std::time::Duration;
use chrono::Local;

pub struct WgetOutput {
    quiet: bool,
    verbose: bool,
    show_progress: bool,
    progress_bar: Option<ProgressBar>,
}

impl WgetOutput {
    pub fn new(quiet: bool, verbose: bool, show_progress: bool) -> Self {
        Self {
            quiet,
            verbose,
            show_progress,
            progress_bar: None,
        }
    }

    /// Print wget-style connection message
    pub fn print_connecting(&self, url: &str, host: &str, port: u16) {
        if !self.quiet {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            println!("--{}--  {}", timestamp, url);
            println!("Resolving {}... ", host);
            println!("Connecting to {}:{}... connected.", host, port);
        }
    }

    /// Print HTTP request sent
    pub fn print_http_request(&self) {
        if self.verbose && !self.quiet {
            println!("HTTP request sent, awaiting response... ");
        }
    }

    /// Print HTTP response status
    pub fn print_http_response(&self, status: u16, status_text: &str) {
        if !self.quiet {
            if status == 200 {
                println!("{} {}", status, status_text);
            } else {
                println!("{} {}", status, status_text);
            }
        }
    }

    /// Print content length and type
    pub fn print_content_info(&self, length: Option<u64>, content_type: Option<&str>) {
        if !self.quiet {
            if let Some(len) = length {
                print!("Length: {} ({})", len, format_bytes(len));
            } else {
                print!("Length: unspecified");
            }

            if let Some(ct) = content_type {
                println!(" [{}]", ct);
            } else {
                println!();
            }
        }
    }

    /// Print saving to file message
    pub fn print_saving_to(&self, filename: &str) {
        if !self.quiet {
            println!("Saving to: '{}'", filename);
            println!();
        }
    }

    /// Initialize progress bar for download
    pub fn init_progress(&mut self, total_size: Option<u64>) {
        if self.quiet {
            return;
        }

        let pb = if let Some(size) = total_size {
            ProgressBar::new(size)
        } else {
            ProgressBar::new_spinner()
        };

        // wget-style progress format
        let style = if total_size.is_some() {
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} {bytes_per_sec} eta {eta}")
                .unwrap()
                .progress_chars("=>-")
        } else {
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {bytes} {bytes_per_sec}")
                .unwrap()
        };

        pb.set_style(style);
        self.progress_bar = Some(pb);
    }

    /// Update progress during download
    pub fn update_progress(&self, progress: &ProgressInfo) {
        if let Some(pb) = &self.progress_bar {
            pb.set_position(progress.downloaded);

            // Update message with current stats
            if let Some(total) = progress.total_size {
                let percentage = (progress.downloaded as f64 / total as f64) * 100.0;
                pb.set_message(format!(
                    "{:.1}% {} eta {}",
                    percentage,
                    format_bytes_per_sec(progress.speed),
                    progress.format_eta().unwrap_or_else(|| "--:--".to_string())
                ));
            } else {
                pb.set_message(format!("{}", format_bytes_per_sec(progress.speed)));
            }
        }
    }

    /// Finish progress bar
    pub fn finish_progress(&mut self) {
        if let Some(pb) = self.progress_bar.take() {
            pb.finish_and_clear();
        }
    }

    /// Print download complete message (wget style)
    pub fn print_complete(&self, filename: &str, downloaded: u64, elapsed: Duration) {
        if !self.quiet {
            let elapsed_secs = elapsed.as_secs_f64();
            let speed = if elapsed_secs > 0.0 {
                downloaded as f64 / elapsed_secs
            } else {
                0.0
            };

            println!();
            println!(
                "{} - '{}' saved [{}]",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                filename,
                downloaded
            );
            println!();
        }
    }

    /// Print error message
    pub fn print_error(&self, error: &str) {
        eprintln!("wget-faster: {}", error);
    }

    /// Print warning message
    pub fn print_warning(&self, warning: &str) {
        if !self.quiet {
            eprintln!("wget-faster: {}", warning);
        }
    }

    /// Print info message (verbose mode only)
    pub fn print_info(&self, info: &str) {
        if self.verbose && !self.quiet {
            println!("{}", info);
        }
    }

    /// Print server response headers (for -S option)
    pub fn print_server_headers(&self, headers: &[(String, String)]) {
        if !self.quiet {
            for (name, value) in headers {
                println!("  {}: {}", name, value);
            }
        }
    }

    /// Print spider mode message (for --spider)
    pub fn print_spider_result(&self, url: &str, status: u16, exists: bool) {
        if !self.quiet {
            if exists {
                println!("Spider mode enabled. Check if remote file exists.");
                println!("--{}--  {}", Local::now().format("%Y-%m-%d %H:%M:%S"), url);
                println!("  HTTP {} OK", status);
                println!("Remote file exists.");
            } else {
                println!("Spider mode enabled. Check if remote file exists.");
                println!("--{}--  {}", Local::now().format("%Y-%m-%d %H:%M:%S"), url);
                println!("  HTTP {} Not Found", status);
                println!("Remote file does not exist -- broken link!!!");
            }
        }
    }

    /// Print retry message
    pub fn print_retry(&self, attempt: usize, max_attempts: usize, wait_secs: u64) {
        if !self.quiet {
            println!(
                "Retrying ({}/{})... waiting {} seconds...",
                attempt, max_attempts, wait_secs
            );
        }
    }

    /// Print timestamping comparison result
    pub fn print_timestamping(&self, local_newer: bool, filename: &str) {
        if !self.quiet {
            if local_newer {
                println!(
                    "Server file no newer than local file '{}' -- not retrieving.",
                    filename
                );
            } else {
                println!(
                    "Server file is newer than local file '{}' -- retrieving.",
                    filename
                );
            }
        }
    }

    /// Print continue/resume message
    pub fn print_resume(&self, filename: &str, resume_from: u64) {
        if !self.quiet {
            println!(
                "Continuing in background. Output will be written to '{}'.",
                filename
            );
            println!(
                "Resuming download. Starting at byte position: {}",
                resume_from
            );
        }
    }

    /// Print quota exceeded message
    pub fn print_quota_exceeded(&self, quota: u64) {
        // Always show quota exceeded errors (even in quiet mode)
        eprintln!("Download quota of {} bytes EXCEEDED!", quota);
    }

    /// Print redirected message
    pub fn print_redirect(&self, from: &str, to: &str) {
        if self.verbose && !self.quiet {
            println!("Location: {} [following]", to);
        }
    }
}

/// Format duration in wget style (e.g., "2m 30s", "1h 15m 20s")
pub fn format_duration_wget(duration: Duration) -> String {
    let total_secs = duration.as_secs();

    if total_secs == 0 {
        return "0s".to_string();
    }

    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    let mut parts = Vec::new();

    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{}s", seconds));
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_wget() {
        assert_eq!(format_duration_wget(Duration::from_secs(0)), "0s");
        assert_eq!(format_duration_wget(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration_wget(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration_wget(Duration::from_secs(3661)), "1h 1m 1s");
    }
}
