# CLAUDE.md

## Build Commands

```bash
cargo build --all-features
cargo clippy --all-features
cargo test --all-features
cargo fmt --all
```

### no_std Build (Embedded)

```bash
# no_std + alloc
cargo build --no-default-features --features alloc

# ARM Cortex-M4F bare-metal target
rustup target add thumbv7em-none-eabihf
cargo build --no-default-features --features alloc --target thumbv7em-none-eabihf
```

## Bencodex Specification

- [Bencodex Spec](https://github.com/planetarium/bencodex/blob/main/README.md) (`spec/README.md`)
- [JSON Representation](https://github.com/planetarium/bencodex/blob/main/JSON.md) (`spec/JSON.md`)
