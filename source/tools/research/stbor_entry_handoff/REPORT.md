# Conditional historical MODEL entry handoff

Parent integration: independent review `e03ce8c2` accepted these exact 25
declarations and the separate17-declaration Tree Borrows/9-command compiler
checkpoints at their stated conditional scope. Parent commit `eb7778b5` replaces
29 Python assert gates across checker/summarizer with explicit failure checks;
the conditions are preserved by AST comparison, and all calls/failure raises
remain in both normal and optimized bytecode. Mathematical sources are unchanged.
The fresh parent `-B -O` replay passed normal compilation, individual assumptions,
the282-library recursive kernel and final guards, reproducing both proof objects.
See [parent-entry-replay.json](../mutable_buffer_goal/parent-entry-replay.json).
The controller revision itself has parent validation, not a new independent
review. Historical controller hashes/receipts below retain their original scope.

Use a fresh output root for current replay, leaving completed artifacts read-only:

```sh
STBOR_ENTRY_OUTPUT_ROOT=/tmp/stbor-entry-FRESH \
  python3 -B -O source/tools/research/stbor_entry_handoff/check.py accepted025 accept
```

**25 own declarations normally compiled and recursively kernel checked at
conditional historical MODEL scope: 3 definitions and 22 proved lemmas.** Every
own declaration's `Check` and `Print Assumptions` reports it individually closed.
The fresh combined acceptance run checks exactly 282 modules, matching the
closure extracted from compiled objects, with no exclusions. See status.json
and the complete compact INVENTORY.json / OBJECTS.json / COMMANDS.json.

The implementation's first scope/interface commit is `b560dfbe`; the 13-law
compilation checkpoint is `07aada06`; the composition pretrial source freeze is
`8128de18`; the corrected compiled composition/controller checkpoint is
`4b0b2ae1`. Parent base is `04b51409`. Only this owned directory changed; no
subdelegation, push, PR, Rust rebuild, Infer regression, installation, toolchain
mutation or production edits occurred. Historical paths were treated as labels.
The uncompiled 13-declaration candidate004 freeze is unchanged. The leading
UNVALIDATED comment retained in candidate.v is historical; the checked status
and exact accepted source hash are recorded here and in status.json. The first
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
membership are part of the checked contract. The second operation preserves
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

Parent-reported milestone `7f415a57` subsequently resolves the concrete
current-Miri type-query condition: the unchanged canonical [u8;U64_LEN] and
HMAC [u8;128] have layout10/align1 and128/align1 and are
Unpin/UnsafeUnpin/Freeze; key [u8] has those traits but remains unsized, with no
runtime key restriction. Parent also reports the generic array/slice trait
check and three E0277 controls. This implementation did not inspect or review
that lane or its receipt, import it into Coq, change a model premise, or count
it as permission/native evidence. The parent reports only own compiler checks,
not independently combined acceptance. The successful concrete type query does
not discharge runtime validation, grants, argument copy, context or C/F gaps.

## Trust and unchanged goal

All 25 own declarations report `Closed under the global context`. The actual
282-module imported closure's kernel context lists only Coq proof irrelevance,
dependent functional extensionality, JMeq equality and Eq_rect_eq; none is an
assumption of any own declaration. The separate complete 518-module foundation
context is broader, as recorded below.
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


## Commands, freezes and retained trials

Reproduction commands use the owned controller and private fresh names:

```
python3 -B source/tools/research/stbor_entry_handoff/check.py NEW_TRIAL
python3 -B source/tools/research/stbor_entry_handoff/check.py NEW_ACCEPT accept
python3 -B source/tools/research/stbor_entry_handoff/summarize.py
```

`COMMANDS.json` binds every actual formal argv, clean environment, cwd, exit,
pid/pgid/startticks, log SHA, source freeze and controller identity. It also
indexes every failed trial. `INVENTORY.json` covers all 25 compiler-discovered
constants, with full Check/Print-Assumptions output hashes and a reference to
the full private inventory; the kernel constant set equals the glob inventory.
`OBJECTS.json` lists every module in the exact checked dependency closure and
binds the full private serialized dependency/import map and loaded object map.
Both new `.vo` files were freshly built together in the acceptance directory.
No Coq8.20/Rocq9.2 objects or old-baseline preparation scripts were used.

Each normal run uses pinned Coq8.9.1/OCaml4.07.1, explicit `-coqlib`, the pinned
upstream `-Q .../theories stbor`, and a fresh private `-Q ... handoff`. `TMPDIR`
and cache are private; the environment is constructed from an allowlist and
contains no COQPATH/COQLIB/OCAMLPATH inherited settings. Actual loadpaths are
printed in the private inventory log. HOME is neither changed nor supplied;
the resulting harmless home-directory warnings are retained. Compilation is
serial; the two kernel/guard jobs overlapped, never exceeding two own jobs.
The final acceptance checks the ready/completed receipt hashes, normal
foundation build/kernel receipts and logs, exact tools/maps, every inventoried
foundation loaded source/object and installed runtime plugin binaries before
and after. The completed full installed/build-tree manifest is authenticated;
it is not falsely claimed entirely reread by final acceptance.

Private root: `/tmp/stbor-entry-handoff-20261110/`. The unchanged original
candidate004 source is bound to base Git and trial001; its freeze and original
status remain committed. Trial001–005 failed on syntax/tactics/elaboration;
trial006 normally compiled the 13 corrected laws. Trial007 was stopped during
an overbroad completed-build-tree setup scan before any formal command and
retains its controller and setup-abort receipt; it is not a proof failure or
semantic negative. Trial008–010 failed on a phantom fns binder and two rewrite
elaboration steps. Trial011 normally compiled both modules. Every executed
proof attempt has a durable source freeze, argv, exit and raw log. Early trial
controllers are reconstructible from Git; later controllers are also copied
privately, and the final controller hashes its source once before execution.
The successful broad `accepted013` check covers the original 13 corrected laws
plus all 518 foundation modules (519 modules total); `accepted025` is the fresh combined 25-law check with
its exact 282-module recursive closure. Neither is an independent reviewer.
The inherited foundation's full trust context remains recorded separately.

Raw logs, compiler outputs, binaries and upstream copies remain private. The
compact committed evidence binds their full inventories; no `.vo`, full raw
log or upstream source tree is committed. The final controller exits only
after the repeated foundation guard and product manifest; the evidence
summarizer additionally verifies those finished products and zero exits.
