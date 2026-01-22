use bencodex::Decode;
use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};

/// Format file size in a human-readable format
fn format_size(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

/// Macro for easily adding benchmark data files (scalar)
macro_rules! bench_decode_files {
    ($group:expr, $( $name:literal => $path:literal ),* $(,)?) => {
        $(
            {
                const DATA: &[u8] = include_bytes!($path);
                let size_str = format_size(DATA.len());
                let bench_name = format!("{} ({})", $name, size_str);

                $group.throughput(Throughput::Bytes(DATA.len() as u64));
                $group.bench_function(&bench_name, |b| {
                    b.iter(|| black_box(DATA.to_vec()).decode())
                });
            }
        )*
    };
}

/// Macro for easily adding benchmark data files (SIMD)
#[cfg(feature = "simd")]
macro_rules! bench_decode_files_simd {
    ($group:expr, $( $name:literal => $path:literal ),* $(,)?) => {
        $(
            {
                const DATA: &[u8] = include_bytes!($path);
                let size_str = format_size(DATA.len());
                let bench_name = format!("{} ({})", $name, size_str);

                $group.throughput(Throughput::Bytes(DATA.len() as u64));
                $group.bench_function(&bench_name, |b| {
                    b.iter(|| black_box(DATA.to_vec()).decode_simd())
                });
            }
        )*
    };
}

pub fn decode_scalar(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_scalar");

    bench_decode_files!(group,
        "ncavatar_1" => "../_data/ncavatar_1.bin",
        "ncinventory_1" => "../_data/ncinventory_1.bin",
        "large_random_0" => "../_data/large_random_0.bin",
    );

    group.finish();
}

#[cfg(feature = "simd")]
pub fn decode_simd(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_simd");

    bench_decode_files_simd!(group,
        "ncavatar_1" => "../_data/ncavatar_1.bin",
        "ncinventory_1" => "../_data/ncinventory_1.bin",
        "large_random_0" => "../_data/large_random_0.bin",
    );

    group.finish();
}

#[cfg(feature = "simd")]
criterion_group!(benches, decode_scalar, decode_simd);

#[cfg(not(feature = "simd"))]
criterion_group!(benches, decode_scalar);

criterion_main!(benches);
