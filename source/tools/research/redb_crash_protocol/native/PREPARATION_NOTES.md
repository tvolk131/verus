# Native context registration notes

Before any native build, an exact extracted libc0.2.191 .cargo-checksum.json lookup failed because that file is absent in this cache. No native/verifier trial ran. The exact cached sparse-index libc0.2.191 record instead supplies checksum65cd87c9a6d60536580b316ad985726104730a5151c99845f2490428ef91240a, matching the available archive byte-for-byte. FREEZE.json records that prospective package identity and all actual build inputs. No assertion of an upstream REDB lock or newly downloaded package is made.

The native host profile is Linux/x86_64 with installed Rust1.99.1, original std-only REDB source and actual build scripts. Cargo/private active-graph projection is preparation, not checked Rust-source correspondence or crypto/backend proof. Fault schedules are not yet registered or run.
