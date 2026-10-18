 -  Added `Encode` implementations for `str`, `[u8]`, `[BencodexValue]`, and
    `BencodexKey`, so existing strings, bytes, and lists can be encoded without
    allocating owned input buffers. Added
    `bencodex::to_writer(writer, &value)`, which accepts owned writers, mutable
    references, and writer trait objects while retaining the existing
    `Encode::encode` signature.
