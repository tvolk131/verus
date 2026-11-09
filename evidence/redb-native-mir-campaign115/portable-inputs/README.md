# Preserved local research inputs

`MANIFEST.json` binds the capsule and already-committed source/evidence archives.
The current new-thread entry point is [`RESEARCH_HANDOFF.md`](../../../RESEARCH_HANDOFF.md).
Use `python3 -B source/tools/prepare_native_research_handoff.py verify` from the
repository root; preparation takes all selected inputs from the clone itself.

| Input | Why preserve it? |
| --- | --- |
| `rust-src-1.99.1.tar.xz` | Exact selected 5,863,216-byte library-source package; its locally recorded SHA256 is `e485a88c45db546b73571df7bebfcb4454ba4fb552c492e25fc5b00eeb5bf22d` |
| `vstd156.vir.gz` | Exact historical 33,396,967-byte serialized library artifact, compressed deterministically; raw SHA256 `022fd17a42e801080dcb9593c1085a7832754eb3ded7aeaa4645bf6428453cb8` |
| `SOURCE-BINDING.json`, `SOURCE-INVENTORY.json.gz`, `bind_sources.py`, `rustc-version.txt` | Preserve the source-acquisition observation and its authored helper, not just a pointer into a vanished cache |

The source archive was checked against the preexisting128 record; the raw `.vir`
against170's retained input binding. Other copied provenance is new endpoint
preservation. Gzip uses an empty filename, timestamp0 and compression level9;
the raw-byte identity is authoritative if gzip encoders differ across machines.
Original local inputs were read, not modified. No runtime executable or Cargo
cache is included. The historical helper and path strings are evidence, not
instructions to run against a new host.

The canonical-vstd JSON is already inside `KERNEL170-PARENT-SNAPSHOT.tar.gz`;
it is not synthesized from names or replaced by a new datatype declaration.
The final170 diagnostic consumed that JSON. The opaque `.vir` is retained for
complete historical input preservation, not required to deserialize JSON.

Preparation's only source change is the explicitly recorded runtime-directory
environment lookup in its new copy. Original archives and readable historical
candidates remain immutable. The path adapter is UNCOMPILED/UNREVIEWED and adds
no native/runtime correspondence or proof credit. All18 and G1/G2 remain OPEN;
worker170 remains unsealed and native export REFUSED.

`validation/RESULT.json` and the raw streams retain the Git-only relocation
check, eleven unit tests, tamper/restoration control and existing-output refusal.
They validate packaging, not an independently reproduced compiler build or proof.
