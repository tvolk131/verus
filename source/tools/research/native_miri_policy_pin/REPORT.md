# Exact compiler-revision Miri policy: inspection, not a proof

This pins ten Miri files inside Rust commit
`58cf8f9c35b355199727627d07ae34a4e2846d67`, the same compiler revision as the
frozen native extraction. No separate moving Miri revision is substituted.
The [manifest](pins.json) records exact official URLs, retained paths and byte
hashes. The inspector checks identity/stability against those pins; it does not
prove that the source implements either published model or normative Rust.

## Findings from the pinned source

The [default configuration](https://raw.githubusercontent.com/rust-lang/rust/58cf8f9c35b355199727627d07ae34a4e2846d67/src/tools/miri/src/eval.rs)
selects Stacked Borrows, shallow validation and integer alignment checks. This
is a Miri configuration fact, not a selected alias semantics of the native
compiler. Miri was not compiled or executed in this inspection.

The [machine hook](https://raw.githubusercontent.com/rust-lang/rust/58cf8f9c35b355199727627d07ae34a4e2846d67/src/tools/miri/src/machine.rs)
dispatches retagging only when its borrow tracker is enabled. The
[tracker](https://raw.githubusercontent.com/rust-lang/rust/58cf8f9c35b355199727627d07ae34a4e2846d67/src/tools/miri/src/borrow_tracker/mod.rs)
selects SB or TB, retains/restores the current retag mode, uses nonzero64-bit
tags and distinguishes weak and strong protectors. Finite tag exhaustion is
instrumentation behavior; an unbounded formal tag counter is not an identity
representation proof for this implementation.

The [SB policy](https://raw.githubusercontent.com/rust-lang/rust/58cf8f9c35b355199727627d07ae34a4e2846d67/src/tools/miri/src/borrow_tracker/stacked_borrows/mod.rs)
assigns mutable TwoPhase references shared-write permission without an access
or protector. Regular mutable references take unique/write behavior when both
Unpin queries succeed; FnEntry adds strong protection. Mutable raw creation
also uses shared-write/no-access/no-protector policy, but obtains a fresh tag.
For known concrete parents the no-access grant inserts at the write-compatible
boundary. Wildcard or unknown-bottom cases have separate approximation effects.
Zero-size operations touch no per-byte stacks, yet can change represented
provenance and consume fresh identifiers. None of these branches is assumed
equivalent to the historical artifact by this inspection.

The [TB implementation](https://raw.githubusercontent.com/rust-lang/rust/58cf8f9c35b355199727627d07ae34a4e2846d67/src/tools/miri/src/borrow_tracker/tree_borrows/mod.rs)
does not assign fresh tags when creating raw pointers. Reference permissions
depend on type properties, protection and feature parameters; implicit writes
can select unique permissions for protected inputs. Its zero-size path can
retain or create provenance depending on whether an allocation is identifiable.
The two implementations cannot share an unconditional tag-preservation rule.

## Effect on the proof route

The published SB artifact already has conditional reference-retag progress and
frame lemmas. Its old raw Untagged convention and zero-size retag behavior differ
from this pinned implementation. The unvalidated
[entry candidate](../stbor_entry_handoff/README.md) therefore uses mutable-reference
retag operations only and awaits the matching checked foundation. The current
native raw-reference operation needs its own justified translation or simulation.

The conservative134-declaration byte-stack model is a third semantic choice;
its checked algebra is not a refinement of either implementation. Source entry
must establish current permissions over all lawful original histories; it cannot
define lawful entry by a convenient successful grant. Conditional grants do not
create original/pending C/F observations or a physical reference interpretation.

Next concrete obligations are to relate per-byte stack representation/order,
known concrete parent lookup, tag freshness/renaming, active protectors, range
updates and all failure outcomes to an independently checked semantics. Unknown
bottom/exposed provenance, zero-size and exceptional cases stay explicit. Actual
Rust source typing/entry, MIR operation and phase simulation and C/F transport
remain separate unproved obligations. No new application premise is introduced.

## Reproduction and limits

```sh
python3 -B -O source/tools/research/native_miri_policy_pin/check_sources.py \
  --output /tmp/native-miri-policy-pin-fresh-UNUSED
```

This validates pinned source bytes only. No model theorem, runtime comparison,
native certificate or production fact is accepted. The failed GitHub contents
API retrieval and nonexistent config.rs URL yielded no evidence; eval.rs supplied
the actual configuration. All original application bodies/domains/properties,
normal checks and trust gates are unchanged. Canonical6/0 and full HMAC1/1 are
inherited, not rerun. Native0;44+3/101 open; publication=false; goal incomplete.
