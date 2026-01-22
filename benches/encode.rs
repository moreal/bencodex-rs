use bencodex::{BencodexValue, Decode, Encode};
use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use num_bigint::BigInt;

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

/// Macro for easily adding benchmark data files for encoding
macro_rules! bench_encode_files {
    ($group:expr, $( $name:literal => $path:literal ),* $(,)?) => {
        $(
            {
                const DATA: &[u8] = include_bytes!($path);
                let size_str = format_size(DATA.len());
                let bench_name = format!("{} ({})", $name, size_str);

                // Setup: decode to BencodexValue
                let value: BencodexValue = DATA.to_vec().decode()
                    .expect(concat!("Failed to decode ", $path));

                $group.throughput(Throughput::Bytes(DATA.len() as u64));
                $group.bench_function(&bench_name, |b| {
                    b.iter(|| {
                        let mut buf = Vec::with_capacity(DATA.len());
                        black_box(value.clone()).encode(&mut buf)
                    })
                });
            }
        )*
    };
}

#[cfg(feature = "json")]
macro_rules! bench_to_json_files {
    ($group:expr, $( $name:literal => $path:literal ),* $(,)?) => {
        $(
            {
                const DATA: &[u8] = include_bytes!($path);
                let size_str = format_size(DATA.len());
                let bench_name = format!("{} ({})", $name, size_str);

                // Setup: decode to BencodexValue
                let value: BencodexValue = DATA.to_vec().decode()
                    .expect(concat!("Failed to decode ", $path));

                $group.throughput(Throughput::Bytes(DATA.len() as u64));
                $group.bench_function(&bench_name, |b| {
                    b.iter(|| bencodex::json::to_json(black_box(&value)))
                });
            }
        )*
    };
}

pub fn encode_primitives(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_primitives");

    group.bench_function("null", |b| {
        let mut buf = Vec::new();
        b.iter(|| black_box(BencodexValue::Null).encode(&mut buf));
    });
    group.bench_function("bigint (9223372036854775807)", |b| {
        let mut buf = Vec::new();
        let bigint = BencodexValue::Number(BigInt::from(9223372036854775807i64));
        b.iter(|| black_box(bigint.clone()).encode(&mut buf));
    });
    group.bench_function("boolean (true)", |b| {
        let mut buf = Vec::new();
        b.iter(|| black_box(BencodexValue::Boolean(true)).encode(&mut buf));
    });
    group.bench_function("boolean (false)", |b| {
        let mut buf = Vec::new();
        b.iter(|| black_box(BencodexValue::Boolean(false)).encode(&mut buf));
    });

    group.finish();
}

pub fn encode_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_files");

    bench_encode_files!(group,
        "ncavatar_1" => "../_data/ncavatar_1.bin",
        "ncinventory_1" => "../_data/ncinventory_1.bin",
        "large_random_0" => "../_data/large_random_0.bin",
    );

    group.finish();
}

#[cfg(feature = "json")]
pub fn encode_to_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_to_json");

    bench_to_json_files!(group,
        "ncavatar_1" => "../_data/ncavatar_1.bin",
        "ncinventory_1" => "../_data/ncinventory_1.bin",
        "large_random_0" => "../_data/large_random_0.bin",
    );

    group.finish();
}

#[cfg(feature = "json")]
criterion_group!(benches, encode_primitives, encode_files, encode_to_json);

#[cfg(not(feature = "json"))]
criterion_group!(benches, encode_primitives, encode_files);

criterion_main!(benches);
