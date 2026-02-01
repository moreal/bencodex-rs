use bencodex::{BencodexDictionary, BencodexKey, BencodexList, BencodexValue, Encode};
use num_bigint::BigInt;
use rand::distr::Alphanumeric;
use rand::prelude::*;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

const DEFAULT_OUTPUT: &str = "_data/large_random_40mb.bin";
const DEFAULT_SEED: u64 = 42;
const MAX_DEPTH: usize = 5;

struct GeneratorConfig {
    binary_chunks: usize,
    binary_max_size: usize,
    number_lists: usize,
    numbers_per_list: usize,
    nested_entries: usize,
    text_entries: usize,
    text_max_size: usize,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            binary_chunks: 400,
            binary_max_size: 50_000,
            number_lists: 60,
            numbers_per_list: 10_000,
            nested_entries: 100,
            text_entries: 600,
            text_max_size: 50_000,
        }
    }
}

fn random_text(rng: &mut impl Rng, max_len: usize) -> String {
    let len = rng.random_range(1..=max_len);
    rng.sample_iter(Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn random_binary(rng: &mut impl Rng, max_len: usize) -> Vec<u8> {
    let len = rng.random_range(1..=max_len);
    (0..len).map(|_| rng.random()).collect()
}

fn random_number(rng: &mut impl Rng) -> BigInt {
    let bytes: [u8; 8] = rng.random();
    BigInt::from_signed_bytes_le(&bytes)
}

fn random_key(rng: &mut impl Rng) -> BencodexKey<'static> {
    if rng.random_bool(0.7) {
        BencodexKey::Text(Cow::Owned(random_text(rng, 32)))
    } else {
        BencodexKey::Binary(Cow::Owned(random_binary(rng, 32)))
    }
}

fn random_primitive(rng: &mut impl Rng) -> BencodexValue<'static> {
    match rng.random_range(0..5) {
        0 => BencodexValue::Null,
        1 => BencodexValue::Boolean(rng.random()),
        2 => BencodexValue::Number(random_number(rng)),
        3 => BencodexValue::Text(Cow::Owned(random_text(rng, 100))),
        _ => BencodexValue::Binary(Cow::Owned(random_binary(rng, 1000))),
    }
}

fn random_value(rng: &mut impl Rng, depth: usize) -> BencodexValue<'static> {
    if depth >= MAX_DEPTH {
        return random_primitive(rng);
    }

    match rng.random_range(0..7) {
        0..5 => random_primitive(rng),
        5 => {
            let len = rng.random_range(1..=10);
            let list: BencodexList = (0..len).map(|_| random_value(rng, depth + 1)).collect();
            BencodexValue::List(list)
        }
        _ => {
            let len = rng.random_range(1..=10);
            let dict: BencodexDictionary = (0..len)
                .map(|_| (random_key(rng), random_value(rng, depth + 1)))
                .collect();
            BencodexValue::Dictionary(dict)
        }
    }
}

fn generate_large_value(rng: &mut impl Rng, config: &GeneratorConfig) -> BencodexValue<'static> {
    let mut dict = BTreeMap::new();

    for i in 0..config.binary_chunks {
        let key = BencodexKey::Text(Cow::Owned(format!("binary_chunk_{i}")));
        let value = BencodexValue::Binary(Cow::Owned(random_binary(rng, config.binary_max_size)));
        dict.insert(key, value);
    }

    for i in 0..config.number_lists {
        let key = BencodexKey::Text(Cow::Owned(format!("number_list_{i}")));
        let list: BencodexList = (0..config.numbers_per_list)
            .map(|_| BencodexValue::Number(random_number(rng)))
            .collect();
        dict.insert(key, BencodexValue::List(list));
    }

    for i in 0..config.nested_entries {
        let key = BencodexKey::Text(Cow::Owned(format!("nested_{i}")));
        dict.insert(key, random_value(rng, 0));
    }

    for i in 0..config.text_entries {
        let key = BencodexKey::Text(Cow::Owned(format!("text_{i}")));
        let value = BencodexValue::Text(Cow::Owned(random_text(rng, config.text_max_size)));
        dict.insert(key, value);
    }

    BencodexValue::Dictionary(dict)
}

fn parse_args() -> (PathBuf, u64) {
    let args: Vec<String> = std::env::args().collect();

    let output_path = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT));

    let seed = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_SEED);

    (output_path, seed)
}

fn format_size(bytes: usize) -> String {
    let mb = bytes as f64 / (1024.0 * 1024.0);
    format!("{mb:.2} MB ({bytes} bytes)")
}

fn main() {
    let (output_path, seed) = parse_args();

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create output directory");
    }

    println!("Generating random Bencodex data...");
    println!("  Output: {}", output_path.display());
    println!("  Seed: {seed}");

    let mut rng = StdRng::seed_from_u64(seed);
    let config = GeneratorConfig::default();
    let value = generate_large_value(&mut rng, &config);

    println!("Encoding value...");
    let mut buf = Vec::new();
    value.encode(&mut buf).expect("Failed to encode value");

    println!("Encoded size: {}", format_size(buf.len()));

    let mut file = File::create(&output_path).expect("Failed to create output file");
    file.write_all(&buf).expect("Failed to write to file");

    println!("Done! File written to: {}", output_path.display());
}
