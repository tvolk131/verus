# Checked native-source reborrow MODEL

Implements the minimal root bb0[3] over existing Storage.state H, with explicit
fresh child/byte grants/history, local4 assignment and control advance to bb0[4].
Normal coqc, complete134-declaration closed assumptions inventory, recursive
coqchk and pure parser controls pass. See [REPORT](REPORT.md),
[typed interface/source rules](INTERFACE.md) and [results](results.json).

```
python3 -B source/tools/research/native_source_reborrow/check.py \
  --freeze source/tools/research/native_source_reborrow/integration001-freeze.json \
  --output /tmp/fresh-native-source-reborrow-replay
```

Output must be fresh. Dependencies/tools are pinned completed read-only artifacts;
only new modules compile privately. The current integration replay uses the
unchanged mathematical candidates and immutable completed receipts; the historical
worker freeze remains preserved. [Parent matching evidence](integration-results.json)
records134 closed declarations and identical mathematical objects. The
[independent review](../../../../evidence/source-reborrow-model-review/REPORT.md)
accepts the exact conditional MODEL and current replay. Global grant-tag
freshness additionally uses `registered`; `ready` alone does not establish it.

This is a conservative authored byte-stack rule, not an implementation of Rust,
Stacked Borrows or Tree Borrows. Actual Rust interpretation, coverage of all
lawful inputs, source certificate and original C/F remain false/open. Native0,
44+3/101 obligations open, publication=false, whole goal incomplete. Generic10
is MODEL-only, never an actual canonical-source certificate. Full HMAC1/1
unproved and the original full domains/independent acceptance remain unchanged.
