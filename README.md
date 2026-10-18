# bencodex-rs

[![build](https://github.com/bencodex/bencodex-rs/actions/workflows/build.yaml/badge.svg)](https://github.com/bencodex/bencodex-rs/actions/workflows/build.yaml) [![codecov](https://codecov.io/gh/moreal/bencodex-rs/graph/badge.svg?token=H0FWUZ2ZF2)](https://codecov.io/gh/moreal/bencodex-rs) [![Docs-rs](https://docs.rs/bencodex-rs/badge.svg)](https://docs.rs/bencodex-rs/latest/)

The [Rust] implementation of [Bencodex].

- **Correctness** - Implement Bencodex spec and passed tests with its testsuites.
- **[Bencodex JSON]** - Support encoding Bencodex to JSON and decoding JSON to Bencodex.
- **Feature flags** - Optional standard-library, JSON, CLI, and SIMD support.
- **`no_std` support** - Can be used in embedded environments with `alloc`.

[Rust]: https://rust-lang.org/
[Bencodex]: https://bencodex.org/

## Quick start

These snippets use `?` inside a function that returns `Result`.

```rust
use bencodex::{BencodexValue, decode_borrowed, to_writer};

let mut bytes = Vec::new();
to_writer(&mut bytes, "hello")?;
assert_eq!(bytes, b"u5:hello");
let value = decode_borrowed(&bytes)?;
assert_eq!(value, BencodexValue::from("hello"));
let owned = value.into_owned(); // Can outlive bytes.
assert_eq!(owned, BencodexValue::from("hello"));
```

Scalar and SIMD decoders read the first value and ignore trailing bytes.
They accept some noncanonical encodings (including leading zeroes and unordered
dictionary keys), so decoding is not a canonical-format validation step.
Invalid lengths and duplicate dictionary keys return errors.

See the [API assessment](docs/api-guidelines.md) for the Rust API Guidelines
review and [release notes](CHANGES.md) for changes and migration guidance.

한국어 설명: [API 가이드라인 평가](docs/api-guidelines.ko.md) · [변경 기록 작성 안내](docs/changelog.ko.md)

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | Yes | Enables standard library support |
| `alloc` | Yes (via `std`) | Compatibility flag; allocation is required even without this flag |
| `json` | No | Enables Bencodex JSON encoding/decoding |
| `json-cli` | No | Enables CLI tool for JSON conversion |
| `simd` | No | Enables SIMD-accelerated decoding |
| `test` | No | Legacy switch enabling the optional base64 dependency; not required to run tests |

## SIMD Decoding

The `simd` feature provides SIMD-accelerated decoding for improved performance on large data. This feature requires `std` and is not available in `no_std` environments.

**Supported architectures:**
- x86_64: SSE4.2 and AVX2 (runtime detection)
- AArch64: NEON

```toml
[dependencies]
bencodex-rs = { version = "<VERSION>", features = ["simd"] }
```

```rust
use bencodex::Decode;

// Using trait method
let data = b"i42e".to_vec();
let value = data.decode_simd()?;

// Or using the function directly
use bencodex::simd::decode_simd;
let value = decode_simd(b"li1ei2ei3ee")?;
```

## `no_std` Support

This crate supports `no_std` environments with the `alloc` crate. To use in a `no_std` environment:

```toml
[dependencies]
bencodex-rs = { version = "<VERSION>", default-features = false, features = ["alloc"] }
```

**Notes:**
- The minimum supported Rust version is 1.88 for all feature configurations.
- `bencodex::io::{Write, Error}` exposes the writer API in both `std` and `no_std` builds. With `std`, these are standard-library re-exports; without it, they are the crate’s minimal writer trait and opaque error type.
- The `json` feature requires `std` and is not available in `no_std` environments.

## Bencodex JSON feature

bencodex-rs implements [Bencodex JSON] feature, encoding and decoding both.

To use Bencodex JSON feature, you should enable `json` feature.

```toml
bencodex-rs = { version = "<VERSION>", features = ["json"] }
```

[Bencodex JSON]: https://github.com/planetarium/bencodex/blob/main/JSON.md

### Encoding to JSON

To encode from Bencodex to JSON, you can use `to_json` function.

```rust
use bencodex::{ BencodexValue, json::to_json };

let json = to_json(&BencodexValue::Null)?;
println!("{}", json);
```

There are two ways to encode `BencodexValue::Binary` type, `Hex` and `Base64`. You can choose one way with `bencodex::json::BinaryEncoding`. And you can pass it with `bencodex::json::JsonEncodeOptions` to `bencodex::json::to_json_with_options`.

```rust
use bencodex::BencodexValue;
use bencodex::json::{ BinaryEncoding, JsonEncodeOptions, to_json_with_options };

let json = to_json_with_options(&BencodexValue::Null, JsonEncodeOptions {
  binary_encoding: BinaryEncoding::Base64,
})?;
println!("{}", json);
```

### Decoding from JSON

To decode from JSON to Bencodex, you can use `from_json_string` and `from_json` function.

```rust
// from_json_string
use bencodex::{ BencodexValue, json::from_json_string };

let value = from_json_string("null")?;
assert_eq!(value, BencodexValue::Null);
```

```rust
// from_json
use serde_json::from_str;
use bencodex::{ BencodexValue, json::from_json };

let json = from_str("null")?;
let value = from_json(&json)?;
assert_eq!(value, BencodexValue::Null);
```

### CLI Tool


Also, it provides a CLI tool to encode from Bencodex to JSON and to decode from JSON to Bencodex. You can install it with `json-cli` feature like the below line:

```bash
cargo install bencodex-rs --features json-cli
```

For improved decoding performance, you can also enable SIMD acceleration:

```bash
cargo install bencodex-rs --features json-cli,simd
```

You can use like the below:

```bash
# encode
$ echo -n 'n' | bencodex
null
$ echo -n 'i123e' | bencodex
"123"
$ echo -n '1:\x12' | bencodex
"0x12"
$ echo -n '1:\x12' | bencodex --base64
"b64:Eg=="

# decode
$ echo -n '"123"' | bencodex -d
123
$ echo -n 'null' | bencodex -d
n
```
