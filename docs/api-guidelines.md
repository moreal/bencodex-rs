# Rust API Guidelines assessment

[한국어](api-guidelines.ko.md)

Reviewed on 2026-10-18 against the official [Rust API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html), including the
[interoperability](https://rust-lang.github.io/api-guidelines/interoperability.html),
[documentation](https://rust-lang.github.io/api-guidelines/documentation.html),
[flexibility](https://rust-lang.github.io/api-guidelines/flexibility.html), and
[future-proofing](https://rust-lang.github.io/api-guidelines/future-proofing.html)
chapters. These are design recommendations whose applicability depends on the
API; this assessment does not claim every checklist item applies or is complete.

The team split review across values/conversions, encoding/I/O, JSON, and SIMD,
then integrated the results with scalar decoding, crate documentation, metadata,
and an independent review. The review covers all feature-gated public library
items. It does not redesign the CLI or the Bencodex format.

## Findings and changes

| Guideline | Initial finding | Result |
| --- | --- | --- |
| C-COMMON-TRAITS, C-DEBUG | Values lacked `Eq`/`Hash`, keys lacked `Hash`, JSON options lacked `Debug` and comparison traits. | Added lawful derives. Borrowed and owned data compare/hash identically; binary and text remain distinct. |
| C-CONV-TRAITS, C-CONV | Missing wide/pointer-sized integer and `BigInt` conversions; no direct key-to-value conversion. | Added lossless `From` implementations. Consuming conversions preserve ownership and avoid copies. Existing `into_owned` naming matches its ownership semantics. |
| C-GOOD-ERR | JSON errors could not be propagated into general application errors; scalar `Display` was just `Debug`. | JSON errors implement `Display`/`Error`; scalar messages explain the invalid input or the token and byte offset. Existing error variants are retained. |
| C-RW-VALUE, C-GENERIC | Encoding required an owned string/vector and a mutable-reference writer entry point. | Added borrowed `Encode` implementations and `to_writer(W, &T)`. Owned writers, mutable references, and `&mut dyn Write` work in both standard and allocation-only builds. |
| C-CALLER-CONTROL | Borrowing and allocation behavior were insufficiently documented. | Documented borrowed payloads, container/integer allocations, consuming ownership conversions, and owned decoding. Callers choose where to copy. |
| C-HIDDEN, C-STRUCT-PRIVATE, C-NEWTYPE-HIDE | SIMD exposed CPU backends, structural indices, and parser machinery as public API. | Internal SIMD modules are private. The two existing `decode_simd` entry paths remain public. No wrapper type is needed for an index that is entirely internal. |
| C-CRATE-DOC, C-EXAMPLE, C-QUESTION-MARK, C-LINK | Crate overview was absent; examples ignored errors or were skipped. | Added linked overview/features/ownership guidance, executable examples using `?`, and documentation for reachable public items. |
| C-FAILURE | Encoding partial-output semantics and decoding failures were unclear. | Added error sections, writer flushing/partial-output contracts, and honest prefix/canonical-form limitations. CPU-specific unsafe internals also document their safety conditions. |
| C-VALIDATE | Duplicate keys caused a scalar panic or SIMD data replacement; very large string lengths overflowed. | Both decoders reject duplicate keys and check string ranges before slicing. Regression tests first reproduced these failures. |
| C-METADATA | Categories and a correct compiler requirement were absent; homepage duplicated repository. | Added categories and Rust 1.88, removed redundant homepage, enabled JSON/SIMD in hosted documentation. |
| C-RELNOTES | No crate release notes or migration guidance. | Adopted Sacho: topic-based fragments in `changes.d/` produce `CHANGES.md`, linked from README/crate docs. Historical release tagging is outside this change. |

`io` is now public so all types needed for downstream `Encode` and writer
implementations can be named. With `std`, it re-exports standard I/O; without
`std`, it provides the minimal allocation-only writer interface and an opaque
error. This also fixes an API usability gap beyond the named checklist items.

## Existing compliance and applicability

| Checklist area | Assessment |
| --- | --- |
| Naming: C-CASE, C-GETTER, C-ITER, C-ITER-TY, C-WORD-ORDER | Public names follow Rust casing and conversion conventions. Collections are standard aliases, so their iterator/getter naming comes from `Vec`/`BTreeMap`; no custom iterator types are exported. |
| C-COLLECT | `BencodexList` and `BencodexDictionary` inherit `FromIterator`, `Extend`, and iteration from their standard collection types. |
| C-SEND-SYNC | Public data, errors, and options use thread-safe component types and retain automatic `Send`/`Sync` implementations. No internal shared mutable state is exposed. |
| C-NUM-FMT | The crate does not define a binary numeric wrapper; arbitrary integers use `num_bigint::BigInt`. Not applicable to tagged binary/text payloads. |
| Macros: C-EVOCATIVE, C-MACRO-ATTR, C-ANYWHERE, C-MACRO-VIS, C-MACRO-TY | No public macros. Internal conversion-generation macros are not API. |
| Predictability: C-SMART-PTR, C-CONV-SPECIFIC, C-METHOD, C-NO-OUT, C-OVERLOAD, C-DEREF, C-CTOR | Conversions live on the concrete value/key types; no custom smart pointers, dereference coercions, overloaded operators, or data out-parameters. Writer arguments are streaming destinations. No constructor wrapper is needed around standard aliases or enum variants. |
| Type safety: C-NEWTYPE, C-CUSTOM-TYPE, C-BITFLAG, C-BUILDER | Tagged values/keys and `BinaryEncoding` distinguish meaning. No boolean mode flag, flag set, or complex constructor needs a newtype, bitflags, or builder. |
| Dependability: C-DTOR-FAIL, C-DTOR-BLOCK | No custom destructors; encoding does not implicitly flush writers. |
| Debuggability: C-DEBUG-NONEMPTY | Public data/option/error debug output includes variants, contents, or fields; the allocation-only error includes its type name. |
| Future proofing: C-STRUCT-BOUNDS, C-SEALED | Data declarations add no duplicated generic bounds. `Encode`, `Decode`, and allocation-only `Write` are intentional downstream extension points, so sealing them would defeat their purpose. |
| Necessities: C-STABLE, C-PERMISSIVE | Crate and direct dependencies use permissive licenses. This is a pre-1.0 crate exposing pre-1.0 `num-bigint` types, so the stable-public-dependency recommendation is a future release concern. |

## Deliberate exceptions and remaining work

- **C-SERDE:** Public values/keys do not directly implement `Serialize` or
  `Deserialize`. The optional JSON API uses an internal Serde adapter for the
  Bencodex JSON representation. A generic Serde wire representation and borrowing
  contract need a separate design; blindly deriving would change the format.
- **C-OBJECT:** `Encode` retains its generic writer method and is not dyn
  compatible. The public value enum supports heterogeneous values; `to_writer`
  accepts writer trait objects. `Decode` consumes its implementor and likewise
  is not a useful erased decoding interface. No dispatch redesign was needed.
- **C-STRUCT-PRIVATE:** `JsonEncodeOptions::binary_encoding` remains public.
  This is deliberate, transparent one-field configuration; a builder/private
  field would add ceremony without protecting an invariant. Adding future
  fields would still require compatibility planning.
- **C-FEATURE:** `std`, `alloc`, `json`, `json-cli`, and `simd` describe capabilities.
  The existing `test` feature is a legacy dependency-enabling switch, and `alloc`
  is a compatibility flag: allocation is required even when it is disabled.
  These are documented rather than removed in an unrelated feature migration.
- **C-CONV-TRAITS:** `From<u8>` would overlap `From<Vec<u8>>` (binary) and generic
  `From<Vec<T>>` (list). Existing binary/list meaning is preserved. Use an
  explicit numeric variant or widen a byte before conversion.
- **C-INTERMEDIATE / C-VALIDATE:** Decoders still read the first value, discard
  the consumed position, ignore trailing bytes, and accept some noncanonical
  forms. A consumed-length/strict-validation API is a separate addition. Deep
  nesting is recursive and has no configurable resource limit. SIMD structural
  offsets are 32-bit; inputs larger than `u32::MAX` are unsupported.

## Verification

Consumer tests cover ownership-sensitive hashing and conversions, external
writer/encoder implementations, borrowed inputs, partial writes, writer trait
objects, application error propagation, JSON options, duplicate keys, and
length-overflow errors. The suite also retains specification fixtures and
feature-gated property tests. Rustdoc examples are executable.

CI checks all-feature and allocation-only builds/tests, formatting, Clippy,
documentation warnings, and the declared Rust 1.88 compiler. Local validation
uses the repository's pinned Rust 1.92 toolchain and also checks Rust 1.88.
Native SIMD execution is checked on AArch64; other architectures retain CI
coverage. Breaking changes and migration paths are in [CHANGES.md](../CHANGES.md).
Changes are authored as [Sacho fragments](../changes.d/) and validated in CI;
see [CONTRIBUTING.md](../CONTRIBUTING.md) for the workflow.

Local verification results:

| Check | Result |
| --- | --- |
| `cargo test --all-features --locked --offline` | 134 tests passed, including 33 rustdoc examples. |
| `cargo test --no-default-features --features alloc --locked --offline` | 73 tests passed, including 21 rustdoc examples. |
| README examples via `rustdoc --test` against the all-feature build | 6 examples passed. |
| Clippy with `--all-targets -- -D warnings`, all features and no default features | Passed. |
| `cargo doc --no-deps` with `RUSTDOCFLAGS="-D warnings"`, all features and allocation-only | Passed. |
| Rust 1.88 `cargo check`, all features and no default features | Passed. |
| Allocation-only check for `thumbv7em-none-eabihf` | Passed. |
| All-feature library check for `x86_64-unknown-linux-gnu` | Passed; cross-compiled, not executed locally. |
| Formatting and whitespace checks | Passed. |

Independent review found no actionable correctness issues. Its stale SIMD module
tree documentation finding was corrected before the final documentation review.
