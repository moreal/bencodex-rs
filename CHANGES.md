bencodex-rs changes
===================

Unreleased
----------

To be released.

 -  Added `Display` and `std::error::Error` to `JsonDecodeError`, allowing JSON
    decoding failures to propagate into application error types with `?`. JSON
    encoding options and binary encoding choices now support debugging,
    comparison, and hashing; JSON error categories also support copying,
    equality, and hashing. Existing JSON representations and default binary
    encoding are unchanged.
 -  Added `Encode` implementations for `str`, `[u8]`, `[BencodexValue]`, and
    `BencodexKey`, so existing strings, bytes, and lists can be encoded without
    allocating owned input buffers. Added
    `bencodex::to_writer(writer, &value)`, which accepts owned writers, mutable
    references, and writer trait objects while retaining the existing
    `Encode::encode` signature.
 -  Added content-based `Eq` and `Hash` for `BencodexValue`, `Hash` for
    `BencodexKey`, and lossless value conversions from `u128`, `i128`, `usize`,
    `isize`, and `BigInt`. Added conversion from a dictionary key to a value
    that preserves borrowed data and moves owned buffers without copying.
    Existing byte vectors still convert to binary values, while other supported
    element vectors convert to lists.
 -  Added public `bencodex::io::{Write, Error}` types so applications can
    implement writers and `Encode` without the standard library. With `std`
    enabled, these names re-export the standard I/O types; without it, they
    provide a minimal writer trait and opaque error. Mutable references to
    `no_std` writers, including trait objects, can also be passed to
    `to_writer`.
 -  Changed `DecodeError` display output to concise messages, including the
    offending byte and its offset for unexpected tokens. Match error variants
    rather than parsing message strings; existing error variants are retained.
 -  Declared Rust 1.88 as the minimum supported compiler for every feature
    configuration. The previous README's Rust 1.81 claim did not account for
    edition 2024 and let-chain syntax already used by the library.
 -  Fixed scalar and SIMD decoding of duplicate dictionary keys to return
    `DecodeError` instead of panicking or silently replacing the earlier value.
    Binary and text lengths that overflow a slice boundary now also return an
    error instead of panicking. Decoders continue to read the first value and
    ignore trailing bytes; successful decoding does not establish canonical
    encoding.
 -  Removed the public implementation modules
    `bencodex::codec::simd::{arch, number, stage1, stage2, structural}` and
    their parser, index, and CPU backend types. Replace direct use of these
    internals with `bencodex::simd::decode_simd` or
    `bencodex::codec::simd::decode_simd`; both decoding function signatures are
    unchanged.
