use bencodex::{BencodexValue, Encode};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use num_bigint::BigInt;

pub fn encode(c: &mut Criterion) {
    c.bench_function("encode null", |b| {
        let mut buf = Vec::new();
        b.iter(|| black_box(BencodexValue::Null).encode(&mut buf));
    });
    c.bench_function("encode bigint (9223372036854775807)", |b| {
        let mut buf = Vec::new();
        let bigint = BencodexValue::Number(BigInt::from(9223372036854775807i64));
        b.iter(|| black_box(bigint.clone()).encode(&mut buf));
    });
    c.bench_function("encode boolean (true)", |b| {
        let mut buf = Vec::new();
        b.iter(|| black_box(BencodexValue::Boolean(true)).encode(&mut buf));
    });
    c.bench_function("encode boolean (false)", |b| {
        let mut buf = Vec::new();
        b.iter(|| black_box(BencodexValue::Boolean(true)).encode(&mut buf));
    });
}

criterion_group!(benches, encode);
criterion_main!(benches);
