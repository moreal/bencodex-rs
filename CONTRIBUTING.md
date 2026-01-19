# Build

You can build and test the code with [cargo] command.

If you want to build for only bytes <-> Bencodex:

```
cargo build
```

If you want to build JSON feature too:

```
cargo build --features JSON
```

If you want to build JSON CLI tool too:

```
cargo build --features json-cli
```

# Test

If you want to test:

```
cargo test --features test
```

If you want to test JSON-related tests too:

```
cargo test --features test,json
```

# Fuzz Test

To run the SIMD decoder fuzzer tests:

```
cargo test --all-features simd_decode
```

To run with more test cases:

```
PROPTEST_CASES=10000 cargo test --all-features simd_decode
```

To see detailed output on failure (including shrinking results):

```
cargo test --all-features simd_decode -- --nocapture
```

# Format

```
cargo fmt
cargo clippy
```

[cargo]: https://github.com/rust-lang/cargo/
