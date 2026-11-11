# Source and dataset citations

Coordinates refer to review base `778c5e6a2a048e4a776be65ea354f73da4de10ba`.
SHA256s for every file below are in INPUTS.json. No raw source/MIR copies are
committed. The MIR report uses precise operational pseudocode, not a new ordinary
function mirror or independent proof object.

| ID | Exact own source coordinates | Evidence |
|---|---|---|
| S1 | Pinned `src/tools/miri/src/borrow_tracker/mod.rs:30–45`, exact private path/hash from own `source/tools/research/native_miri_policy_pin/pins.json` | Seven derives and private field; minimal declaration: `pub struct BorTag(NonZero<u64>);`. Declaration-only SHA256 `0107890f5ffee6b94a4179c0351a4ff121e0e61b9886067e94370a65391e9abd`. |
| S2 | `source/vstd/std_specs/nonzero.rs:38–47,91–102,114–123` | Opaque external type; uninterpreted View; existing get returns `n@`, no invariant opening/no unwind. Only PartialOrdSpecImpl is supplied. |
| S3 | `source/vstd/std_specs/cmp.rs:149–160` | Existing Ord external trait extension; guarantee guarded by obeys_cmp_spec. |
| S4 | `source/vstd/std_specs/num.rs:71–91,601` | Unsigned numeric OrdSpecImpl, true obeys, existing primitive Ord method model; u64 macro instantiation. This file is essential, not just cmp.rs. |
| S5 | `source/rust_verify/src/checked_foreign.rs:43–174,220–290,426–482` | Exact From<bool>u64 resolver, single IntToInt bool cast, complete actual VIR comparison and direct call registration. |
| S6 | `source/rust_verify/src/checked_foreign.rs:24–37,512–605,608–662` | Current implicit bool certificate, fresh semantic/MIR/compiler/core/target identity, single-input Function canonicalization, full-domain contract check. |
| S7 | `source/rust_verify/src/query.rs:375–408`; `source/rust_verify/src/rust_to_vir_func.rs:2345–2354`; `source/rust_verify/src/rust_to_vir.rs:703`; `source/rust_verify/src/fn_call_to_vir.rs:367–391` | BodyBackedForeign enters ordinary function pipeline; direct source-call helper mapping. |
| S8 | `source/builtin_macros/src/infer_derive_ops.rs:24–40,184–188`; `source/vir/src/infer/wrapper_compare.rs:110–130,161–205,208–232`; `source/rust_verify/src/automatic_derive.rs:28–37` | Macro NonZero PartialEq gate, scalar request constraints, ordinary derived-method eligibility, genuine carrier recognition; default ordering derives ignored. |
| S9 | `source/rust_verify/src/import_export.rs:13–18,65–83,184–214,230–260,405–443` | Schema/policy, model artifact fingerprinting, current bool-only fresh import resolution, transitive foreign guard, private pruning and independent evidence retention. |
| S10 | `source/vir/src/infer.rs:1071–1104,1126–1149`; `source/vir/src/infer/wrapper_compare.rs:10–38`; `source/rust_verify/src/verifier.rs:3102–3150` | Approved models and inferred-function call collection are filtered; ordinary helper dependency closure requires explicit attention. |
| S11 | `source/rust_verify/src/verifier.rs:1864–1871,2913–2934,2978–3004,3702–3710` | Record exact successful normal helper body queries, detect before pruning, retain checked Function binding, require erasure checks, reject missing queries. |
| S12 | `source/rust_verify/src/verifier.rs:3015–3025,3076–3099,3731–3753`; `source/rust_verify/src/driver.rs:323–362`; `source/rust_verify/src/import_export.rs:390–403` | Full scope and ordinary checks, deferred interface export, global error gate, final erased Rust compiler gate. |
| D1 | `/tmp/nonzero-ordering-mir-x1sbsnfo/inspect/stdout`: sections 1–141 (`partial`),142–281 (`total`),282–433 (derived partial_cmp),434–596 (derived cmp),597–853 (carrier),854–948 (get),949–1080 (primitive) | Complete WHOLE_MIR sections, including typed locals/CFG/body properties; private file SHA256 `1350a691732ff2888175b581635e68942d582075a75e1efa230b261a283b3b8e`. Seven section hashes/fingerprints are in own MIR.json and INPUTS.json. |
| D2 | `source/tools/research/nonzero_ordering_goal/{REPORT.md,PROPOSAL.md,INVENTORY.md,BASELINE.json,MIR.json,SEAL.json,run.py,audit.py,mir_inventory.py,finish.py}` | Retained diagnostic/baseline scope. SEAL contains exactly 168 artifact names and hashes; audit main writes BASELINE and was not run. |
| D3 | `source/tools/research/mutable_buffer_goal/parent-nonzero-ordering-baseline.json`; `/tmp/verus-infer-buffer-cycle-20261109/nonzero-ordering-parent-auth/receipt.json`; `/tmp/verus-infer-buffer-cycle-20261109/auth-nonzero-ordering.py` | Accepted baseline scope and normal/optimized parent authentication are inherited, not replayed or promoted to ordering proof. Controller read only, never run against its hardcoded parent root. |

Historical worker `cd120a4a+d7512005` maps to root `92e26872+eaa86203`; parent
metadata is `778c5e6a`. These are provenance labels. This review read only its
own checkout/Git and explicitly named sealed files/pinned source/completed
authentication tools. No receipt path was followed into live parent targets.
