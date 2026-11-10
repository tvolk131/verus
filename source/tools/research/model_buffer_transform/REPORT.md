# Checked generic memory transformation

Trial007 passed normal compilation and recursive kernel checking for all eight
definitions/lemmas. Every assumption inventory is closed. `STATUS.json` binds
the accepted receipt; the exact pretrial freeze is `checked-loop-freeze.json`.
Independent review is pending. Earlier failures were proof-script development,
never semantic negative evidence; their receipts and sources remain indexed.

`wp_zip_transform` executes actual existing SC reads of the pad and key heads,
a binary-operation call, an SC store to that pad head, and recursion at the
next addresses. Its operation WP is an explicit proof obligation for every
pair of byte inputs; no XOR summary or no-write rule is assumed. The standard
WP frame rule preserves the retained cells across that call. Exactly one £1
write credit is consumed per iteration, from the initial £n.

For any n bounded by both input lengths, the result returns the entire
`transform_prefix n f pad key` interval and the entire original key interval.
The pure length and drop laws prove unchanged pad length and the untouched
suffix. Full separating resources are retained, not just copied observations.
Other resources are preserved by framing. Choosing n=min(length pad,length key)
covers empty and oversized keys. A pad of length 128 is a specialization; no
zero-initialization or allocation-base restriction appears.

The internal WP takes interval resources. It is not an added requires clause
on the original Rust writer; the checked heap initialization can supply model
resources in a later public composition. The recursive model expression is
not claimed to be the unchanged native Zip body. An actual byte-XOR operation,
its source instruction correspondence, native loop correspondence, original
loan recovery and C/F import remain required. No full HMAC or engine automation
success follows from this conditional model law. Native/publication remain 0/false.

Replay using a fresh private output directory:

```sh
python3 -B -O source/tools/research/model_buffer_transform/check.py \
  --freeze source/tools/research/model_buffer_transform/checked-loop-freeze.json \
  --output /tmp/fresh-model-buffer-transform
```

Only existing semantics and the pinned foundation are used. All new proof
sources, dependencies and outputs remain guarded through recursive checking.
There are no new axioms, Program obligations, trusted contracts or production
changes. The operation contract may prove only partial correctness; this law
does not claim arbitrary supplied operations terminate.
