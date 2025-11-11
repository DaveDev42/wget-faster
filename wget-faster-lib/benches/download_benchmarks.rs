use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use wget_faster_lib::{Downloader, DownloadConfig};
use mockito::Server;
use tokio::runtime::Runtime;

/// Benchmark sequential downloads of different file sizes
fn bench_sequential_downloads(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("sequential_downloads");

    // Test different file sizes
    let sizes = vec![
        ("1KB", 1024),
        ("10KB", 10 * 1024),
        ("100KB", 100 * 1024),
        ("1MB", 1024 * 1024),
    ];

    for (name, size) in sizes {
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, &size| {
            b.to_async(&rt).iter(|| async move {
                let mut server = Server::new_async().await;
                let data = vec![0u8; size];

                let mock = server
                    .mock("GET", "/file")
                    .with_status(200)
                    .with_header("content-type", "application/octet-stream")
                    .with_header("content-length", &size.to_string())
                    .with_body(&data)
                    .create_async()
                    .await;

                let config = DownloadConfig::default();
                let downloader = Downloader::new(config).unwrap();
                let url = format!("{}/file", server.url());

                let result = downloader.download_to_memory(black_box(&url)).await;

                mock.assert_async().await;
                assert!(result.is_ok());
            });
        });
    }

    group.finish();
}

/// Benchmark parallel downloads (when enabled)
fn bench_parallel_downloads(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("parallel_downloads");

    // Only test sizes that would trigger parallel downloads (> 10MB)
    let sizes = vec![
        ("10MB", 10 * 1024 * 1024),
        ("20MB", 20 * 1024 * 1024),
    ];

    for (name, size) in sizes {
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, &size| {
            b.to_async(&rt).iter(|| async move {
                let mut server = Server::new_async().await;
                let data = vec![0u8; size];

                // Mock server needs to support Range requests
                let mock = server
                    .mock("HEAD", "/file")
                    .with_status(200)
                    .with_header("accept-ranges", "bytes")
                    .with_header("content-length", &size.to_string())
                    .create_async()
                    .await;

                let mock_get = server
                    .mock("GET", "/file")
                    .with_status(200)
                    .with_header("content-type", "application/octet-stream")
                    .with_header("content-length", &size.to_string())
                    .with_header("accept-ranges", "bytes")
                    .with_body(&data)
                    .create_async()
                    .await;

                let config = DownloadConfig {
                    parallel_chunks: 8,
                    ..Default::default()
                };
                let downloader = Downloader::new(config).unwrap();
                let url = format!("{}/file", server.url());

                let result = downloader.download_to_memory(black_box(&url)).await;

                mock.assert_async().await;
                mock_get.assert_async().await;
                assert!(result.is_ok());
            });
        });
    }

    group.finish();
}

/// Benchmark progress tracking overhead
fn bench_progress_tracking(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("progress_tracking");

    let size = 1024 * 1024; // 1MB

    group.bench_function("without_progress", |b| {
        b.to_async(&rt).iter(|| async {
            let mut server = Server::new_async().await;
            let data = vec![0u8; size];

            let mock = server
                .mock("GET", "/file")
                .with_status(200)
                .with_header("content-type", "application/octet-stream")
                .with_body(&data)
                .create_async()
                .await;

            let config = DownloadConfig::default();
            let downloader = Downloader::new(config).unwrap();
            let url = format!("{}/file", server.url());

            let result = downloader.download_to_memory(black_box(&url)).await;

            mock.assert_async().await;
            assert!(result.is_ok());
        });
    });

    group.bench_function("with_progress", |b| {
        b.to_async(&rt).iter(|| async {
            let mut server = Server::new_async().await;
            let data = vec![0u8; size];

            let mock = server
                .mock("GET", "/file")
                .with_status(200)
                .with_header("content-type", "application/octet-stream")
                .with_body(&data)
                .create_async()
                .await;

            let config = DownloadConfig::default();
            let downloader = Downloader::new(config).unwrap();
            let url = format!("{}/file", server.url());

            let progress = std::sync::Arc::new(|_info: wget_faster_lib::ProgressInfo| {
                // Simulate minimal progress handling
            });

            let result = downloader
                .download_to_memory_with_progress(black_box(&url), Some(progress))
                .await;

            mock.assert_async().await;
            assert!(result.is_ok());
        });
    });

    group.finish();
}

/// Benchmark different chunk configurations
fn bench_chunk_sizes(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("chunk_sizes");

    let size = 10 * 1024 * 1024; // 10MB
    let chunk_configs = vec![4, 8, 16, 32];

    for chunks in chunk_configs {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}chunks", chunks)),
            &chunks,
            |b, &chunks| {
                b.to_async(&rt).iter(|| async move {
                    let mut server = Server::new_async().await;
                    let data = vec![0u8; size];

                    let mock_head = server
                        .mock("HEAD", "/file")
                        .with_status(200)
                        .with_header("accept-ranges", "bytes")
                        .with_header("content-length", &size.to_string())
                        .create_async()
                        .await;

                    let mock_get = server
                        .mock("GET", "/file")
                        .with_status(200)
                        .with_header("content-type", "application/octet-stream")
                        .with_header("content-length", &size.to_string())
                        .with_header("accept-ranges", "bytes")
                        .with_body(&data)
                        .create_async()
                        .await;

                    let config = DownloadConfig {
                        parallel_chunks: chunks,
                        ..Default::default()
                    };
                    let downloader = Downloader::new(config).unwrap();
                    let url = format!("{}/file", server.url());

                    let result = downloader.download_to_memory(black_box(&url)).await;

                    mock_head.assert_async().await;
                    mock_get.assert_async().await;
                    assert!(result.is_ok());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_sequential_downloads,
    bench_parallel_downloads,
    bench_progress_tracking,
    bench_chunk_sizes
);
criterion_main!(benches);
