# Historical source-only review integration

The original complete review is immutable Git snapshot `a11e3b05`, under
`NEW/evidence/published-entry-source-review/`. The worker interpreted the word
NEW as a literal path prefix. This integration places its findings under the
normal evidence directory and preserves their original bytes.

[ORIGIN.json](ORIGIN.json) binds every original artifact. The two full upstream
Rust files and three historical ad-hoc scripts are retained in Git history,
removing more than2,500 source lines from the live review directory. For example:

```sh
git show a11e3b05:NEW/evidence/published-entry-source-review/call.rs
```

The original commands and `HISTORICAL_SHA256SUMS` describe that historical
snapshot, not a current-directory replay recipe. Their checkout paths are labels.
Do not follow them into an old checkout or overwrite a completed receipt. The
official primary-source URLs and hashes remain in the retained manifest.

This review inspected the12-declaration candidate at `136d8f82`; later candidate
changes need their own validation. It accepted no compiled proof, kernel result,
source certificate or Rust correspondence. The earliest missing operation is
the FnEntry argument copy into callee `_1`, followed by an entry/prefix frame.
The [missing sequents](MISSING-LEMMA.md) remain proof obligations. Native0,
publication=false, whole goal incomplete.
