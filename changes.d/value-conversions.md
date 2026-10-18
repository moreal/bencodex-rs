 -  Added content-based `Eq` and `Hash` for `BencodexValue`, `Hash` for
    `BencodexKey`, and lossless value conversions from `u128`, `i128`, `usize`,
    `isize`, and `BigInt`. Added conversion from a dictionary key to a value
    that preserves borrowed data and moves owned buffers without copying.
    Existing byte vectors still convert to binary values, while other supported
    element vectors convert to lists.
