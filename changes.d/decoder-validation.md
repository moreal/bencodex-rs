 -  Fixed scalar and SIMD decoding of duplicate dictionary keys to return
    `DecodeError` instead of panicking or silently replacing the earlier value.
    Binary and text lengths that overflow a slice boundary now also return an
    error instead of panicking. Decoders continue to read the first value and
    ignore trailing bytes; successful decoding does not establish canonical
    encoding.
