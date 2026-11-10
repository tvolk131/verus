# Executable storage-prefix MODEL: current integration replay

The current replay pins the unchanged domain/completion requirements and retains the full current charter snapshot during execution. Documentary progress updates do not invalidate the requirements pin.

The historical worker report and final freeze remain unchanged. They describe the delivered checkpoint; use the integration freeze below with the current checker. Mathematical candidates are unchanged.

```sh
python3 -B -O source/tools/research/native_source_storage/check.py \
  --freeze source/tools/research/native_source_storage/integration001-freeze.json \
  --output /tmp/native_source_storage-integration-replay-UNUSED
```

[Integration receipt and limits](integration-results.json) bind the matching source/object hashes and complete checks. [Historical worker report](REPORT.md) preserves the original trial.

The [independent review](../../../../evidence/source-storage-model-review/REPORT.md)
accepts the exact MODEL scope and current replay. Native correctness, source
interpretation and full HMAC remain unproved. Publication=false; goal incomplete.
