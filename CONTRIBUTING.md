# Build

You can build and test the code with [cargo] command.

If you want to build for only bytes <-> Bencodex:

```
cargo build
```

If you want to build JSON feature too:

```
cargo build --features json
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

# Changelog

User-visible changes are documented with [Sacho](https://sacho.dev/).
See the [Korean workflow guide](docs/changelog.ko.md) for a detailed explanation.

Install [mise](https://mise.jdx.dev/), then run `mise trust` and `mise install`
to activate the Sacho version pinned in `mise.toml`. Alternatively, install
Sacho directly with `cargo install sacho --version 0.3.2 --locked`.

Create a topic-based fragment with `mise exec -- sacho add topic-name`, then
edit the printed file under `changes.d/`. Describe the public behavior and any
action an upgrading user must take. Use one top-level unordered list; one
item per fragment is usual. Update an existing fragment when the same unreleased
feature changes again. Do not edit the generated Unreleased region in
`CHANGES.md` directly.

Before committing, run:

```sh
mise run changelog:fmt
mise run changelog:preview
mise run changelog:check
```

The equivalent direct commands are `sacho fmt`, `sacho preview`, and
`sacho check`. Keep the source change and its fragment in the same commit.
On pull requests, CI checks each commit after the base revision for changes to
`src/**` or `Cargo.toml`. A follow-up commit adding the fragment does not cover
an earlier commit. For maintenance with no user-visible change, a commit may
explicitly opt out with a `Changelog: none` trailer.

For a local comparison of committed changes, run
`mise exec -- sacho check --base <revision>`; use `--staged` to check the index
before committing. Plain `sacho check` validates fragments and generated output
without checking whether source changes have matching fragments.

No next release version is selected by this setup. When preparing a release,
run `mise exec -- sacho release <version>` after reviewing the preview. It
consumes the fragments and freezes the released section in `CHANGES.md`.
Commit and tag that state, then open the next development cycle separately
with `mise exec -- sacho next <next-version>`. These commands do not publish
the crate; version updates and publishing remain part of the release process.
