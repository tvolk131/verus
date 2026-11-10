# Reproduce and verify

Run from this checkout's repository root. Use `python3 -B`; no dependency,
source, policy or cache changes are needed. Existing accepted trial008 and all
prerequisite artifacts remain read only. Archived repository paths appearing
inside receipts are rebased into this checkout by `reproduce.py`.

Verify committed evidence files:

```sh
sha256sum -c evidence/mutable-parent-recovery-review/SHA256SUMS
```

Reauthenticate the completed review, all five assumptions, real logs, all 887
retained input identities, accepted protocol/child/recovery receipts and source/
object identities, official baseline and five pinned libraries:

```sh
python3 -B evidence/mutable-parent-recovery-review/reproduce.py \
  --receipt /tmp/verus-infer-buffer-cycle-20261109/mutable-parent-recovery-review-9b759adb/receipt.json \
  > /tmp/verus-infer-buffer-cycle-20261109/mutable-parent-recovery-review-9b759adb/audit-repeat.json
cmp evidence/mutable-parent-recovery-review/audit-results.json \
  /tmp/verus-infer-buffer-cycle-20261109/mutable-parent-recovery-review-9b759adb/audit-repeat.json
```

Fresh proof replay and authentication (choose an output that does not yet exist):

```sh
python3 -B evidence/mutable-parent-recovery-review/reproduce.py \
  --replay /tmp/verus-infer-buffer-cycle-20261109/mutable-parent-recovery-review-new001 \
  > /tmp/mutable-parent-recovery-review-new001-summary.json
```

The replay invokes the **unchanged committed checker** with trial008-freeze.
It performs normal `coqc` for resource_protocol/child/initialized, compiles the
five-constant assumption inventory, then performs full recursive `coqchk` on
all three modules plus inventory. It writes source copies, logs, compiled
objects and the receipt only in the newly created private output. The new
summary has new receipt identity/timings but must retain identical object and
source hashes and the same assumptions. No Rust/HMAC or excluded experiment is
part of either command. Existing completed dependency libraries are prerequisites
rather than rebuilt during this review.

Review output receipt SHA256:
`9dae078caf9e16daaa0a89bf86576d4433684ad988c790899f563150e44e8bee`.

The first evidence-audit script attempt encountered the semantic library's
generated inventory name `AssumptionInventory.v` instead of `Assumptions.v`.
The reproducer was corrected to recognize both generated inventory filenames;
that setup failure is not proof evidence. The one fresh mathematical replay
itself completed successfully without a source or checker edit.
