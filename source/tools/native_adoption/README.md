# Native adoption integration example

This standalone Cargo crate uses actual derive_more and codec derives. It keeps
ordinary native implementations while explicitly excluding unused Sum/Encode/
Decode from verification. Those exclusions supply no trusted contracts. The
selected arithmetic, assignment derives and both conversion directions are checked
against the real generated bodies before inferred callers can use them.

Build the matching release compiler, macros and vstd, then run from the Verus
repository root:

```sh
python3 source/tools/check_native_adoption.py --output /tmp/native-adoption-check
```

The runner checks a fresh provider, repeated candidate reuse, a false application
goal, attempted verified calls of excluded methods, no-verify/filtered publication
gates and a restored positive build. It also runs native Rust tests which exercise
the retained Sum and codec implementations and the conversions. The default
native test toolchain is 1.85.1; verification uses the fork's pinned toolchain.
The runner invokes Cargo from the repository root and clears compiler-development
Rust flags for native tests, so the embedded example does not accidentally inherit
`source/.cargo`'s unstable development configuration on the older native compiler.

Full stdout/stderr, command arguments, identities and structured scope/certificate
records are retained below the chosen output directory. Native tests are bounded
tests; exact helper certificates and the ordinary roundtrip requirement are
verification results. A successful selected proof does not certify the excluded
serializers or Sum implementation.
