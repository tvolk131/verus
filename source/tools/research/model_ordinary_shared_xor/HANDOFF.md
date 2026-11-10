# Checked integration handoff

Loadpath: `-Q /tmp/model-ordinary-shared-xor-trial-008/proofs ModelOrdinarySharedXor`.
The authenticated receipt is
`/tmp/model-ordinary-shared-xor-trial-008/receipt.json`, SHA256
`79b5452cb7e3dee9cf69ea6582eda68e6fc5ec3c31ab1e37dc6ac38c563b75ab`.
Use the complete source/object pin maps in `results.json`; dependency loadpaths
and receipt pins are in `dependency-pins.json`. The final freeze is byte-identical
to this successful trial's pretrial freeze. No proof source changed after it.

Import `From ModelOrdinarySharedXor Require Import adapter.` Relevant APIs:

- `reader_na.shared_key_read_na`: ordinary E/L, Cap, Frame; 3*(g+1) per read.
- `reader_na.shared_key_reader_na_session_prepared`: ordinary E/L + Cap → fresh
  R/G, G guard, Na reader, same Cap, closure to original L; no prepare credits.
- `reader_na.shared_key_reader_na_session`: raw view plus g+1 prepare credits.
- `adapter.sc_session_to_na`: consume old SC R/G via old closure; frame any
  assertion and open fresh Na session from the already prepared Cap.
- `adapter.raw_pair_reader_na_context_wp` and `raw_pair_reader_na_observed`:
  frozen fixed ModelPairReader setup output adapted to genuine Na reader at
  d=1,g=0, with PadRef/Lpad/outside Frame and exact same Outer. Their callback
  shape is otherwise the SC callback shape, with `key_reader_na`. Setup remains
  2+k; public model adequacy supplies all credits internally.
- `xor_na.zip_transform_na`: the frozen loop tree, with only three order changes.
- `xor_na.wp_shared_xor_loop_na`: generic start/count, budget count*3*(g+1),
  original base q, advancing start, exact same Cap/R/Frame after the loop.
- `xor_na.wp_shared_xor_na_min` / `wp_shared_xor_na_128`: exact xor_result and
  prefix/suffix/length/range facts for all finite keys.
- `xor_na.wp_shared_xor_na_zero`: Cap separately, no Reader or memory access,
  including p=q; no first-cell ownership premise.

Preserve G and closure outside the loop (or in Frame), then close the returned R
with G to recover ordinary Lkey for whole-key observation. No old SC R is
returned by the adapter. It does not require or produce an exclusive key vector.
Prepared-Cap reuse is proved; a new raw view still needs g+1 preparation credits.

All 21 declarations (including one persistent instance) and all 84 inventory
helpers passed normal coqc/Print Assumptions and full recursive coqchk without
exclusions. Seven failed setup/development attempts are retained in attempts.json
and their private receipts; none is semantic evidence. Printed proof-term section
hashes are diagnostic output hashes and may reflect Coq printing depth limits;
recursive kernel checking is complete.

No integrated initialized writer/final observer proof is claimed here; root owns
that composition. No native or concurrent-equivalence result: native0, 44+3/101
open, publication false, goal incomplete. Canonical6/0 and HMAC1/1 inherited,
not rerun. No original Rust body/domain, old SC function, baseline, compiler,
vstd, helper, parent or sibling checkout was edited. No subdelegates, push or PR.
