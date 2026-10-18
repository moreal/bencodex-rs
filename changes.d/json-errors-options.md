 -  Added `Display` and `std::error::Error` to `JsonDecodeError`, allowing JSON
    decoding failures to propagate into application error types with `?`. JSON
    encoding options and binary encoding choices now support debugging,
    comparison, and hashing; JSON error categories also support copying,
    equality, and hashing. Existing JSON representations and default binary
    encoding are unchanged.
