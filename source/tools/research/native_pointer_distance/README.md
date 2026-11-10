# Conditional pointer-distance component: current integration replay

The current pointer replay individually inventories all 154 generated inspection helpers, all 77 source declarations, and discovers the complete module inventories with Coq. The 231 total declarations include 203 constants; inductives and constructors are tracked separately.

The historical worker report and final freeze remain unchanged. They describe the delivered checkpoint; use the integration freeze below with the current checker. Mathematical candidates are unchanged.

```sh
python3 -B -O source/tools/research/native_pointer_distance/check.py \
  --freeze source/tools/research/native_pointer_distance/integration003-freeze.json \
  --output /tmp/native_pointer_distance-integration-replay-UNUSED
```

[Integration receipt and limits](integration-results.json) bind the matching source/object hashes and complete checks. [Historical worker report](REPORT.md) preserves the original trial.

Native correctness, source interpretation and full HMAC remain unproved. Publication=false; goal incomplete. Independent acceptance of the latest replay changes remains pending.
