# Conditional historical MODEL entry handoff

Implementation in progress; see status.json for chronological acceptance state.
The uncompiled 13-declaration candidate004 freeze is unchanged. The first
successful normal compilation was private trial006. Trials001–005 were syntax,
tactic and type-elaboration failures, not semantic negatives; their exact
sources, argv, exits and raw logs remain private. The original Git author
configuration failure was a setup error before the scope commit, corrected with
command-local identity, without changing Git configuration.

The target composition starts with published `valid_block`, WF, positive model
size and a live top call. This is a conditional MODEL writable block, not a
Rust client requires. It assumes neither caller-top, pool_safe, successful
retag nor a future defined execution. `FnEntry` normalizes to a fresh Unique
head protected by the current call, then published `TwoPhase` reborrows the
returned tag to SharedReadWrite. Both events use the actual published
`retag`/`head_step`, with the exact returned tag as the next operand. Full
heap/call/next-call preservation, WF, outside-span frames and active protector
membership are the intended checked contract. The second operation preserves
the complete normalized stack as its tail. The first can remove incompatible
unprotected younger items, so there is no blanket preservation claim for all
caller items. The original scalar-vector theorem expresses heap stuttering;
no proof of the HMAC byte transformation follows.

Generic positive n includes 10 and 128 model scalar locations. These sizes have
no independently proved relation to Rust byte extent, array layout or physical
allocation/subobject identity. For n=0, the historical retag simply returns the
old tag and unchanged stacks/counters even at an unallocated/dangling model
location. This historical scope is separate from positive-size progress and
from current Rust behavior. Old Untagged raw tags, unbounded natural-number
tags and the explicit `check_bounds` TODO prevent silent transfer to current
Miri or pinned Rust1.99.1 semantics.

## Source obligations still absent

The delivered source-only review covered the earlier 12 declarations, not the
13-declaration candidate or this new composition. Its proposed missing MODEL
sequent must be distinguished from source coverage. The parent retains actual
source interpretation. The exact missing source prefix remains:

1. Bind the original arbitrary initialized exclusive array and function/MIR
   instance to actual `caller_arg_copy`, the callee `Place::from(Local(1))`,
   `&mut [u8;128]`, allocation, offset, provenance and original C/F observations.
2. Derive incoming model `valid_block` and WF from independent actual entry
   permissions/history, including protector compatibility. A successful grant,
   top-head or future defined execution cannot define lawful inputs. Unknown
   bottom/wildcard provenance requires a covering refinement or richer relation,
   not a narrower native domain.
3. Cover the `copy_op_allow_transmute(&caller_arg_copy, &callee_arg)` under
   `with_retag_mode(FnEntry)`, including carrier reads/writes, type/ABI checks,
   reference validation, layout/non-null/alignment/bounds/allocation/mutability,
   per-byte write-grant checks and installation of the resulting pointer in
   this callee `_1`.
4. Frame that binding/head/protector/initialized referent through all remaining
   arguments, in-place and return-carrier protections, frame-init hooks and
   actual root `StorageLive(2/3/4)` to bb0[3]. A protected item need not remain
   head through an arbitrary intervening operation.
5. Relate every bb0[3] outcome: `eval_place(*_1,true)`, carrier read,
   `force_allocation`, `to_ref`, TwoPhase validation/retag, write to `_4`,
   local-write hooks and successful PC advance. Cover errors/stops/panics/UB,
   partial effects, ABI-specific unwind/cleanup and protector release. Neither
   no-unwind nor original defined execution may be assumed. Relate finite tag
   exhaustion rather than adding a client budget.
6. Extend through activation, unsize/raw/getter/writer/call/normal/cleanup exit
   operations and bind the original C/F observations and all lawful inputs.

The MODEL result is a relation among published expressions and stacks. It has
no actual Rust local-carrier, PC, source Cfg, source operation or C/F instance.
Hashes/source identity and conditional model algebra do not supply that bridge.

## Trust and unchanged goal

New concrete declarations must each report `Closed under the global context`.
Full dependency kernel context is a separate inventory, not attributed to all
new lemmas: inherited JMeq equality, classical logic, proof irrelevance,
dependent functional extensionality, Eq_rect_eq, Equations funext and the
inherited admitted `Equations.Type.Subterm.FixWf_unfold`, plus six sealed Iris
interface fields backed by the upstream checked module implementation. No new
assumptions, admissions, unsafe mode or disabled check is authorized.

Native correspondences ZERO; all44 minimal +3 phase simulations /101 writer
effects OPEN; publication=false; goal_complete=false. Canonical unsigned varint
for every u64/arbitrary initialized exclusive [u8;10], exact body/domain and
predicate stays inherited6/0. Independently frozen bitcoin_hashes0.14.0 HMAC
pad XOR for arbitrary initialized [u8;128], EVERY valid key length including0
and>128, BOTH XOR prefix and untouched suffix stays inherited1/1 UNPROVED.
No app-local mechanical proof/call/snapshot accepted fixture was introduced.
Actual same checked assistance, normal checking and fresh combined independent
acceptance remain required. This implementation worker cannot self-certify
that independent acceptance. No Rust rebuild/full Infer regression was run.
