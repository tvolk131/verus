# Executable native extraction; native_bridge_proved=false

This standalone rustc_private prototype queries the **actual monomorphized MIR**
of the frozen local three-line writer and recursively resolves concrete callees.
It does not install facts, change production code, or prove the frozen full
HMAC writer theorem. The original executable loop remains the input; neither
an indexed replacement nor a hand-written native trace is used as evidence.

The first guarded replay (`extraction-receipt.json`) builds and runs with Rust
1.99.1, normal compiler analysis and overflow checks. It extracts 25 concrete
instances and 401 statements/terminators, including the specialized Zip new/next,
both indexed getters, constructors, size calculations, and drop glue actually
encountered. Every MIR local has its instantiated type, target layout and
`needs_drop` query; every block has source locations, operations, successors and
resolved call edges. `mir_sha256` binds the queried body; identity hashes do not
certify its semantics. Foreign generic MIR is available through `instance_mir`.

Extraction is complete as a call inventory, with three explicit semantic leaves:
`ptr_offset_from_unsigned` is an intrinsic without a modeled semantics, and
`panic`/`panic_nounwind_fmt` lack available bodies. These are blockers, never
assumed harmless. Finite instance bounds stop expansion conservatively.

Raw typed MIR and logs remain private under
`/tmp/native-iterator-certificate-20261109-41eec989/dev04/`.
`milestone-extraction.json` commits only the compact inventory. The shared
`infer_experiment.py`/`infer_eval_support.py` guards and receipts run under
Python `-O`. `extraction-freeze.json` predates the accepted build/extraction.
Earlier development compiler/API errors are excluded from accepted evidence.

Reproduce from the repository root:

```sh
python3 -B -O source/tools/research/native_iterator_certificate/replay.py \
  --output /tmp/a-new-private-native-certificate-run
```

The extraction milestone checks **no ownership/refinement theorem**. Remaining
work includes a supported typed operation layer, provenance/bounds/stride
checks, source-connected VC generation, all-slot conservation, native borrow
resolution/drop correspondence, and the unchanged full original theorem for
every initialized pad and every valid key length. No production certificate
publication path exists. Native proof status remains false.
