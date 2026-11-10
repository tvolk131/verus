# Ordinary-memory writer under an existing caller

`caller.caller_xor_na_verified` proves `caller_xor_na_contract` in an existing
typeG and ordinary E/L/I context. It returns the original parent at the same
lifetime, with exact min-length XOR current bytes, untouched suffix and the
parent's original future unchanged. The shared-key view and arbitrary ancestor
Frame remain available. There is no fresh heap/context initialization or ending
of the caller's parent lifetime.

The body uses the existing checked `zip_transform_na byte_xor`. The proof reuses
the accepted SC caller's generic guarded child accessor, result vector and
lifetime lemmas; it replaces only the concrete loop, reader session and credit
index. It does not duplicate the guarded access or byte-algorithm proofs.
All finite n/k and typed byte contents are quantified, including zero, 128 and
key lengths above 128. Byte ranges are derived from the input types, with no
extra range, output-summary, PointsTo or exclusive-key-vector premise.

After child reborrow, the checked Na reader session splits the single ordinary
L into R/G. G guards unique child access while R remains available for reads.
ReaderNa/Cap/R/loop credits explicitly enter the physical callback. The callback
executes the Na loop and returns R; the same child body is closed before R/G
rejoin L, and only then does the generic protocol end the child. The parent
retains its exact incoming future. No future=current equation is assumed.

The explicit MODEL resource precondition includes symbolic credits
`(g+1)+(1+min(n,k)*3*(g+1))`: reader preparation, payload opening and loop work.
These and the lifetime aliveness hypotheses are internal MODEL typing premises,
not new requirements for the original Rust caller, a proved Rust entry rule or
a minimal source execution budget. The public raw-initialized ordinary model
theorem, which supplies credits internally, is a separate composition.

The pretrial interface fixes the model expression and semantic contract. The
only frontend correction adds required notation imports; definitions and the
contract are byte-identical to the initial candidate. The final controller
derives its kernel inventory from the actual module list and rejects duplicate
or extra declarations. No source algorithm, domain or proof condition changed.

## Validation and replay

The accepted trial is `trial003-freeze.json`. Results bind normal compilation,
individual assumptions and proof-term inventories for all five source
declarations, 20 closed inventory helpers and full recursive kernel checking.
Only inherited dependent functional extensionality and `Eq_rect_eq` occur.
The checker authenticates completed source/object/receipt dependencies,
foundation readiness AND its successful durable receipt, all 523 sources and
284 objects, tools/loadpaths/libraries and the GMP pin. Final guards precede
success. Raw outputs remain in the private path recorded by `results.json`.

Trials 001 and 002 are retained setup/controller failures, not semantic
negative evidence: missing notation imports, then a stale module-name regex.
The final replay uses the corrected, newly frozen controller. Independent
acceptance of this five-declaration extension remains pending.

```sh
python3 -B -O source/tools/research/model_caller_ordinary_writer/check.py \
  --freeze source/tools/research/model_caller_ordinary_writer/trial003-freeze.json \
  --output /tmp/model-caller-ordinary-replay-UNUSED
```

MODEL only. Actual Rust input interpretation, source-step validity/erasure,
retag/cleanup and original C/F import remain unproved. Native correspondences
are zero; 44 minimal plus three phase simulations and all 101 writer effects
remain open. Full HMAC1/1 is inherited, not rerun. Publication and overall goal
completion remain false. No production fact, source rewrite, new axiom,
unsafe mode, shared build or push/PR was introduced.
