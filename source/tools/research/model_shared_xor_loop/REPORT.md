# Shared-key XOR loop (MODEL)

The candidate proves the existing `ModelBufferTransform.zip_transform` with
`ModelByteXor.byte_xor`, using checked `ModelSharedByteRead.session.key_reader_read`.
It adds no executable expression, language primitive, axiom or trusted summary.
`shared_xor_result`, `shared_xor_cost` and `shared_xor_contract` are proof data.

`shared_xor_verified` establishes the exact contract for any start/count with
count <= length pad and start+count <= n, where n = length (byte_list n x).
It returns the physical pad containing
`transform_prefix count Z.lxor pad (drop start (byte_list n x))`, the same linear
R, persistent key Cap, and arbitrary separating Frame. The postcondition proves
unchanged pad length, unchanged suffix after count, and byte range. The pad may
contain any initialized bytes and may start at any location.

The induction keeps the original key base q and vector x fixed while start
increases. Its actual SC read is at q+start; `key_reader_read` provides a literal
`byte_list n x !! start = Some z` and byte range. This lookup identifies the
head of `drop start (byte_list n x)` via `drop_S`. The recursive pointer equality
is `(q+start)+1 = q+(S start)`. The loop expression and its pointer increments
are unchanged. The internal operation-parametric helper is discharged with
checked `wp_byte_xor` in `wp_shared_xor_loop`; no operation premise remains in
the concrete XOR interface.

Each iteration splits count*(g+2) into g+1 shared-read credits, one store credit,
and the remaining loop budget. This is an explicit sufficient budget; no
minimality claim is made. Preparation remains outside this law. Reader is
persistent, but every read requires R and returns that same R. R stays spatial:
it is never duplicated, made persistent, discarded or recreated. Cap is a
separate persistent input retained even at count=0. An empty reader law is not
used to manufacture Cap. The checked session produces both Reader and Cap.
Frame stays outside each reader call (which uses the trivial internal frame),
so arbitrary aliases and other resources remain available and are returned.
No exclusive physical key interval is required or extracted.

`wp_shared_xor_min` specializes start=0/count=min(length pad,n). It returns the
imported `xor_result`, whose prefix is pointwise Z.lxor over zip and whose
remaining pad suffix is original. `wp_shared_xor_128` restricts only pad length
to 128 and byte initialization. Thus it covers every initialized [u8;128]
pad, every finite key length (including 0, 128 and >128), and arbitrary key
contents. These are universal theorems, not finite test fixtures.

All WP resources are internal MODEL resources. They are not appended to Rust
application requirements. The parent must obtain Reader/Cap and R from the
checked session, supply the pad loan and credits internally, and close the
initialized loan with the actual new pad before recovering the original parent
and final heap. The committed `model_mutable_writer/INTERFACE.md` describes that
separate Rin/Rout boundary. For that callback, instantiate Frame=True and
reassociate the resources as Rin = R * credits and Rout = R * Cap; Reader and
the independently supplied Cap remain persistent. This changes no application
precondition or ownership token. This directory does not depend on its unfinished
observer, a live parent replay, or the independent pair-input/reader bridge.

`EVIDENCE.json`, `TRUST_INVENTORY.json` and `REPRO.md` carry the final replay,
object, assumption and proof-term hashes, complete declaration inventory,
pretrial freezes and retained trial classifications. Raw logs, objects and
failed trial snapshots remain private in the named /tmp trial directories.
Development/setup failures are excluded from semantic evidence.

This is partial-correctness MODEL WP, not native Rust execution, provenance,
retag, native Zip correspondence, termination, parent final-heap adequacy or C/F
proof. No compiler, vstd, shared helper, original Rust body/domain or production
fact changed. Actual native correspondences remain 0; all 44 minimal and 3
phase simulations / 101 effects remain open. Full HMAC prefix/suffix remains
the inherited frozen 1/1 result, not rerun or improved here. Publication=false
and the overall Infer goal remains incomplete. Parent integration and independent
review remain outstanding.

Accepted trial011 passed normal coqc and full recursive coqchk (no exclusions
or admission flags), followed by all retained-artifact guards. The complete
inventory is 12 source constants plus 48 closed inventory helpers; no new
instances, constructors or obligations. Per-constant semantic assumptions use
only inherited functional extensionality. Recursive dependencies additionally
use inherited Eq_rect_eq; the kernel reports no unsafe principles. Trial010
passed compilation/kernel checking but was excluded as an inventory-controller
ordering failure. Trial004 is the accepted typed milestone; trials001–003 and
005–009 are retained proof-development/setup failures, never semantic negatives.
