 -  Added public `bencodex::io::{Write, Error}` types so applications can
    implement writers and `Encode` without the standard library. With `std`
    enabled, these names re-export the standard I/O types; without it, they
    provide a minimal writer trait and opaque error. Mutable references to
    `no_std` writers, including trait objects, can also be passed to
    `to_writer`.
