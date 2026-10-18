 -  Removed the public implementation modules
    `bencodex::codec::simd::{arch, number, stage1, stage2, structural}` and
    their parser, index, and CPU backend types. Replace direct use of these
    internals with `bencodex::simd::decode_simd` or
    `bencodex::codec::simd::decode_simd`; both decoding function signatures are
    unchanged.
