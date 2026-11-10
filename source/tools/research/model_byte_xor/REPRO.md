# Reproduce without modifying pinned dependencies

From the repository root, use an unused private output directory:

```sh
python3 source/tools/research/model_byte_xor/check.py \
  --freeze source/tools/research/model_byte_xor/trial017-freeze.json \
  --output /tmp/verus-byte-xor-replay-NEW
```

This requires the already-built pinned Coq 8.20.0 foundation and the completed
ModelBufferTransform trial007 paths recorded in the controller and committed
foundation/loop STATUS records. It does not install or rebuild dependencies.
The output directory must not exist and must be outside this repository.
No freeze recreation is necessary for an exact replay. A changed source,
controller, support, external dependency inventory, baseline receipt or retained
loop artifact rejects before a successful result. All commands, their output,
initial/final guards and proof artifacts are retained in the private receipt.
The controller's success status is set only after normal compilation, complete
assumption/proof-term inventories and recursive kernel validation, followed by
all final stability checks. No filesystem substitution/concurrency test is run.

The external dependency digest is over compact sorted JSON mapping resolved
absolute paths to SHA-256, separators `(',', ':')`. The normal source input map
and baseline tool/object maps are checked separately. Per-constant proof-term
hashes bind the UTF-8 section between paired BEGIN/END markers of `Set Printing
All; Print ModelByteXor.byte_xor.<constant>`. They are hashes of the explicit Coq
printing, accompanied by hashes of the kernel-checked `.vo` binary and source.
