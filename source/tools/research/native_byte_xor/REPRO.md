# Normal replay

Use a new private directory that does not exist, outside the checkout:

```sh
python3 source/tools/research/native_byte_xor/check.py \
  --freeze source/tools/research/native_byte_xor/trial008-freeze.json \
  --output /tmp/verus-native-byte-xor-replay-NEW
```

No installation or shared rebuild is performed. Required read-only dependencies
are Coq 8.20.0 / VerusBelt 115a618d in the pinned foundation; the completed
ModelBufferTransform trial007; the completed ModelByteXor trial017; and the exact
source receipt/extraction/native JSON paths in SOURCE.json. The controller checks
both readiness and successful durable-build receipt, all 523 copied sources,
284 baseline objects, tool hashes, loadpaths and the pinned GMP manifest.

The freeze binds source statements, controller, selector, shared supports,
committed interface/source-results/dependency pins and the external dependency
inventory before execution. The replay copies proof sources and JSON snapshots
into the private output, compiles normally using explicit -Q paths, prints every
constant's assumptions and proof term, and runs recursive `coqchk -o` with no
admission flag or dependency exclusion. Stability checks run before commands and
again on input, retained JSON/freeze and proof artifacts before success is set.

Raw diagnostics, complete commands, source/object hashes and assumption/proof-term
inventories remain in the private receipt and logs. Failed development trials
are not semantic evidence. All selector controls modify structured data only in
memory; there are no filesystem substitution/concurrency experiments.
