# Reproducing this review

Use the existing frozen scripts and a fresh private root. Do not use parent fresh receipts as inputs. This review's root is `/tmp/redb-crash-checkpoint-review-20261112-1ca1cae5`.

First reproduce the direct model failure from frozen parent 1ca1cae5 with normal Python (not -O):

```sh
python3 -B source/tools/research/redb_crash_protocol/model/replay.py --label review_direct
```

It fails on CURRENT.md. For the bounded workaround, copy the protocol subtree (excluding existing evidence) and the two CASES-authenticated shared helpers into a private checkout with the same relative paths. Restore only CURRENT.md and WORKER_BRIEFS.md in that private copy from `git show c008326bfed88eab958335412cdc5ec0e9c8130e:source/tools/research/redb_crash_protocol/<file>`, checking CASES charter_sha256 for each. projection.json records the exact before/after hashes. Preserve the current protocol, controls, registrations and scripts byte-for-byte. Do not overwrite the live checkpoint or disable assertions.

Run that private model/replay.py normally once for protocol.rs and then once for each of the eight CONTROLS.json candidates using `--candidate <private-control-path> --expected postcondition --label <unique-label>`. Each call privately copies/authenticates the accepted eight tools and verus-root and uses fresh cache paths. The private controls.py regeneration was checked byte-exact against the frozen controls before replay. Relocate each newly generated model/evidence/redb-model-accepted78-* directory unchanged into private retained storage and hash all products. model-results.json maps each label to its retained path.

The source and native entrypoints ran directly from the frozen checkout, with fresh nonexistent output directories:

```sh
python3 -B source/tools/research/redb_crash_protocol/source/replay.py \
  --output /tmp/FRESH-review-source \
  --cases publication publication_wrong publication_unsafe supported selection serialization backend_sync \
  --validate --imports
python3 -B source/tools/research/redb_crash_protocol/source/audit.py /tmp/FRESH-review-source
python3 -B -O source/tools/research/redb_crash_protocol/native/fault_replay.py \
  --output /tmp/FRESH-review-native
```

Native requires the exact authenticated FAULT_FREEZE prepare2 paths; these are read-only inputs. Python -O is the native report's registered invocation; its require() checks and ArtifactGuard use explicit exceptions, so optimization does not disable them. No prepare.py/Cargo/compiler rebuild is required or authorized by this review. Native reads only specifically frozen parent-path files, checked equal to the local copies. Parent comparison receipts are read afterward only from the three paths in PARENT_VALIDATION.json.

RESULTS.json, source-audit.json and source-authentication.json retain qualified body scopes, hashes, costs and git comparisons. PRIVATE_ARTIFACTS.json binds private retained raw products. Exact copied receipts and diagnostics are in receipts/. This process supports only the partial results and does not complete CP1/CP2 or HMAC/native.
