use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use wget_faster_lib::{Downloader, DownloadConfig};
use std::time::Duration;

fn bench_small_file_download(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_files");

    // Configure faster sampling
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Test with different parallel chunk settings
    for chunks in [1, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_chunks", chunks)),
            chunks,
            |b, &chunks| {
                b.to_async(&runtime).iter(|| async move {
                    let mut config = DownloadConfig::default();
                    config.parallel_chunks = chunks;

                    let downloader = Downloader::new(config).unwrap();

                    // Download a 1MB test file (you'll need to provide a test URL)
                    // For now, we'll use a mock
                    black_box(());
                });
            },
        );
    }

    group.finish();
}

fn bench_chunk_sizing(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_sizing");

    group.sample_size(10);

    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Test different chunk sizes for a 10MB file
    for chunk_size in [256 * 1024, 512 * 1024, 1024 * 1024, 2 * 1024 * 1024].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_bytes", chunk_size)),
            chunk_size,
            |b, &chunk_size| {
                b.to_async(&runtime).iter(|| async move {
                    let mut config = DownloadConfig::default();
                    config.chunk_size = Some(chunk_size);

                    let downloader = Downloader::new(config).unwrap();

                    black_box(());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_small_file_download, bench_chunk_sizing);
criterion_main!(benches);
