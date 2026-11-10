Accepted for the exact executable byte-XOR MODEL scope of commit `648110da`.
No blocking formal-mathematics or evidence-authentication finding. This review
was performed independently from isolated base `3ce5025c`; the reviewed directory
is identical to `648110da`. Only this evidence directory is changed.

The operation in `source/tools/research/model_byte_xor/candidates/byte_xor.v:8`
is finite AST data: `seq 0 256` supplies both table dimensions, and every leaf is
an integer literal labelled by Coq's `Z.lxor` on fixed natural indices. There is
no runtime XOR primitive. `RecV BAnon` has no self reference; the body contains
only the two cases and their literals/argument variables. The substitution and
lookup lemmas (lines 28–62) cover every byte, including zero and 255, without
selected-value premises. No nondeterministic choice or divergent branch is used.

`byte_xor_pure` (line 65) composes one beta step and two case steps into
`PureExec True 3`. In the authenticated Iris language, each pure step includes
availability for every state and determinism over every primitive transition:
empty observations, identical entire state, unique successor, no forks.
`byte_xor_normalizes` (line 86) extracts exactly three actual primitive steps
from that proof for arbitrary `sigma`, with that same `sigma` at every step.
The endpoint is a literal value, so the three-step result cannot hide continued
computation. Neither theorem assumes heap ownership, state validity or credits.
`byte_xor_range` proves the full byte closure, including zero cases separately
from the positive `log2` bounds.

`wp_byte_xor` (line 149) proves the existing
`ModelBufferTransform.transform.binary_operation_spec Z.lxor byte_xor` under
the usual `lrustGS` context, without client heap or credit premises. The pure
lifting rule provides credits to its continuation; the proof discards them.
The authenticated loop's interface and source identity were checked; its
previously accepted eight-constant proof was not independently re-reviewed.

`xor_prefix` is exactly `map (fun (a,b) => Z.lxor a b) (zip xs ys)`;
`xor_result` appends `drop (min (length xs) (length ys)) xs`. The structural
induction equating this result to `transform_prefix` handles both exhaustion
cases. Length, prefix, suffix and byte-range lemmas establish the claimed list
semantics. `wp_xor_memory` (line 157) instantiates the existing physical loop at
that minimum for arbitrary byte vectors, returning the exact result and the
unchanged **whole** key vector. Its permissions for both vectors and one store
credit per iteration remain explicit internal WP resources. The 128 variant
(line 175) adds only `length xs = 128`; no bound or nonemptiness premise is added
to the key. Empty and longer-than-128 keys are therefore included. Range is in
the general WP and the separate range lemma; the specialized WP retains the
length/prefix/suffix facts. No raw-heap adequacy or whole-loop termination
theorem is claimed.

Authentication and replay were successful:

- Trial017 receipt SHA-256 is the required
  `5e663c2d6e8affdf453afbadc3d33e1a6fa5859934ef109cad6925f932766b9e`.
  STATUS, EVIDENCE, TRUST_INVENTORY, the retained freeze, six source/object
  artifacts and all eight retained logs agree. All 26 explicit printed term
  digests were reconstructed, and all 26 assumption sections are closed.
- The pinned baseline-ready record **and** successful durable build receipt
  authenticate the 523 copied sources, 284 objects, loadpaths and four tools.
  The successful version/build/kernel-stage logs also match their recorded
  hashes. A fresh version command reports Coq 8.20.0, compiled with OCaml 4.14.2.
- The independently enumerated external set has 1,269 resolved files and digest
  `f17a2b04d6da609eec8bfd137ca32d8c42a69dab9d770a4074926c620ec38d27`:
  five loop artifacts, 587 Coq files, 653 Coq-core files, four OCaml stub files,
  and 20 GMP files. GMP manifest identity is
  `332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`;
  both retained package archives also match their manifest hashes.
- The immutable trial007 loop receipt has SHA-256
  `687c9f1ab9a09456cf54cecfb8a440386b48e5a425ad7d7223cc92c7f7762aa7`.
  Its committed STATUS binding is itself authenticated by the retained trial017
  guard snapshot and freeze. All four loop source/object artifacts match that
  receipt and snapshot; the checkout loop source matches its accepted copy.
- `check.py:94` authenticates retained JSON bytes and frozen inputs before
  compilation. It removes COQPATH, COQLIB, COQCORELIB, OCAMLPATH,
  CAML_LD_LIBRARY_PATH, LD_PRELOAD and LD_LIBRARY_PATH. Explicit authenticated
  loadpaths and the pinned runner supply the switch/GMP environment. Copied
  candidate and generated helper sources have identity guards. Logged commands
  check inputs before and after execution. Final held-source, product and input
  guards (lines 199–202) precede the proof-success update (line 203); the durable
  receipt wrapper sets success only on normal completion.
- Fresh normal replay in NEW private output
  `/tmp/verus-byte-xor-model-review-df66e1b1-replay-001` passed all four commands.
  All three `.vo` objects and three `.v` sources are byte-identical to trial017;
  all 26 printed proof-term hashes and all 2,098 input-content bindings agree
  after normalizing only the repository prefix in retained JSON values.
  Neither the original worktree paths in that JSON nor any live parent output
  was opened. Full recursive `coqchk -o` checked all 26 candidate declarations
  and 52 marker theorems in **each** helper module. There are no new constructors
  or Program obligations. Only the foundation's inherited functional
  extensionality and Eq_rect_eq appear in the recursive closure; every candidate
  constant is closed. Final independent rehashes also matched the replay inputs
  and products. Compact hashes and command results are in `evidence.json`.
- All 17 retained development receipts, copied candidates and freeze snapshots
  authenticate: trials 001–014 failed compilation (003 records `User interrupt`),
  and 015–017 succeeded with the identical final candidate. These are development
  classifications, not semantic negatives or native results. Nine pure in-memory
  parser/GMP-identity controls passed, including six expected rejections. No
  artifact substitution, concurrency, source-changing or control-driver
  experiment was performed; no installation or shared-baseline write occurred.

Exact replay command, from this isolated repository root:

```sh
python3 -B source/tools/research/model_byte_xor/check.py \
  --freeze source/tools/research/model_byte_xor/trial017-freeze.json \
  --output /tmp/verus-byte-xor-model-review-df66e1b1-replay-001
```

The recorded output already exists; a later replay must use a NEW private path.
No compiled objects, auxiliary files, caches, raw replay logs or copied proof
directories are committed.

Acceptance is conditional on the existing lrust/Iris foundation and explicit
internal WP resources. It establishes no Rust/MIR instruction, incoming loan,
phase/getter, actual source/state or C/F correspondence. Native correspondences
remain 0; 44 minimal operations, three phase obligations and 101 effects remain
open. Canonical remains 6/0, full frozen HMAC prefix AND suffix remains 1/1,
publication is false, and the parent goal remains active. MODEL results cannot
complete that goal.
