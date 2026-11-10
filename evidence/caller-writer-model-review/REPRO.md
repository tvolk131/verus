# Reproduce this review

Use the current checkout and already-installed pinned Rocq toolchain. Do not build Rust/HMAC or modify the implementation, foundation or dependencies. Run with `python3 -B`. Each replay path must be fresh, private, and outside this repository.

The completed implementation's STATUS/REPORT select `source/tools/research/model_caller_writer/final-freeze.json`, not a guessed generic final artifact. The committed `review-freeze.json` was independently created before the review trial and is byte-identical to that selected freeze.

```sh
python3 -B source/tools/research/model_caller_writer/check.py \
  --freeze evidence/caller-writer-model-review/review-freeze.json \
  --output /tmp/caller-writer-model-review-2c4795b5-002
python3 -B evidence/caller-writer-model-review/verify.py \
  --trial /tmp/caller-writer-model-review-2c4795b5-002 --resolution
```

The first command authenticates the mandatory ready marker and successful baseline receipt, all 523 foundation sources/284 objects, tool/runner/GMP pins, eleven completed dependency libraries, all frozen source/controller inputs and the 1,367 external runtime/dependency artifacts. It normally compiles all source and inventory modules, checks all 45 local declarations with full recursive coqchk, and retains every input/product through success. It clears inherited Coq/OCaml/library override variables and uses explicit pinned loadpaths.

The second command independently checks the exact implementation commit, original trial009 receipt and source/object identities; rebases historical source labels onto this checkout without reading the historical worktree; authenticates all selected dependency artifacts; reparses fresh assumptions, terms and kernel declaration inventory; normally compiles the pre-frozen name-resolution diagnostic; and rechecks retained hashes/metadata. It updates only this owned directory's RECEIPT.json, INVENTORY.json and INPUT_HASHES.json; raw output and objects remain private. `--resolution` requires a fresh diagnostic subdirectory. To recheck an existing completed trial, omit that flag:

```sh
python3 -B evidence/caller-writer-model-review/verify.py \
  --trial /tmp/caller-writer-model-review-2c4795b5-001
```

The recorded original review run is `001`; `002` is an example fresh replay. New runs naturally change private receipt/log hashes and timing fields. Re-running verify on `001` is content-stable. The independent pretrial freeze was created using the unchanged controller:

```sh
python3 -B source/tools/research/model_caller_writer/check.py \
  --create-freeze --freeze evidence/caller-writer-model-review/new-review-freeze.json
```

That optional command requires a nonexistent output file; compare its bytes with the committed selected freeze before using it. It is not necessary to reproduce the recorded review.

Verify the committed evidence from this directory with `sha256sum -c HASHES.sha256`. This checks the review evidence, not fresh raw output from a new replay.

All claims remain MODEL only: native 0; 44+3/101 open; HMAC 1/1 open; publication false; goal incomplete.
