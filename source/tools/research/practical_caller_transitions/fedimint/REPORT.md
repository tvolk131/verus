# Fedimint F3 eligibility / F3-B bookkeeping value checkpoint

**Full `Keychain::is_complete` and `ConsensusEngine::validate_signed_session_outcome` remain UNPROVED.** The independently preregistered F3-B replacement proves concrete signature-slot bookkeeping, including the original `add_signature` executable body specialized to the actual `[u8;64]` signature type. It does not prove signature validation, the original iterator's count, known-peer membership, initialization or full external trait integration. Parent milestone completion and NEW independent review remain outstanding.

## Source / runtime / freezes

Parent charter `2914c073c5a962fb83845c35297f8e3848c6862c`. No ancestor AGENTS.md files were present at the inspected exact ancestor paths. Only this new directory was edited. No live parent/sibling work was read. Parent's subsequent P0 notice agrees with the locally measured automatic slot frames; its files were not needed or inspected.

Ordered local preregistrations, all before their respective trials:

1. `8fb5fce2`: original full sources, properties, domain, trust and eligibility adaptations frozen.
2. `5aaddf5c`: full-consumer setup/iterator eligibility retained; actual NodeMap insert/get bookkeeping selected and frozen.
3. `aa8b782b`, `9f55499a`: explicit specification corrections after failed type/frontend attempts, preserving the original baseline. Set::new returns Option; use existing Set::from_finite_type for usize. Mutable-reference postconditions require final(self)/final(map). Executable algorithms/domain/property unchanged.
4. `ea447adf`: preregister actual concrete add_signature body, clone dependency and unchanged original item_count probe before trials.
5. The delivery commit adds ordinary manual proof, imported client, controls and this checkpoint. Original freezes and accepted earlier fixtures/reports remain unchanged.

INPUTS.json authenticates clean Fedimint commit `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`, exact Cargo.lock and contextual sources, and seven crate archives against lock checksums. The real chain is fedimint-aleph-bft0.36.0 -> aleph-bft-types0.13.0 -> aleph-bft-crypto0.9.1. NodeMap is **NodeMap<T>(Vec<Option<T>>)**. Archive SHA `4e99189e0c434af4bb0f4238925df15091962ae843f509561def48ecf36c163d`; node.rs SHA `4dd1a0495537ca73a81a345f92e3bfc16adfeea1ceaf8317d22ef0724ac77f41`; signature.rs SHA `c1695f0620ba6adcce0456649150c7523d5a864e37acbcfb1227e6d95e47ac7d`.

Authenticated COMPLETE private combined78 bundle: `/tmp/fedimint-f3-3ad8038e-auth/bundle`. Receipt SHA `3e04cec7545fd1b44781ba6e8c2447e2109339a9f5257ef7b605d81896a16452`, success/source_commit5ecea5fe and **all eight tool digests** checked before copy and rechecked privately. verus-root retained. Every application/provider/control uses --no-cheating, with ordinary body/lifetime/trait/WF/mode/erasure checks. No ignored root runtime, historical75, build, network, shared Cargo/cache mutation, production/vstd changes, push or PR. No subdelegation. Source files explaining inherited vstd contracts were also compared byte-for-byte to accepted engine commit5ecea5fe (FIDELITY.json).

## What is eligible, and what is not established

| Exact selection | Ordinary | Infer | Interpretation |
|---|---|---|---|
| Full original keychain file with verification attributes | Dependency/setup failure | Same | Standalone file lacks real crate graph/config/crypto dependencies; **zero full-caller body evidence** |
| Full original engine file with verification attributes | Dependency/setup + expression-attribute frontend failure | Same | Whole-file adaptation also reaches unrelated async/loop context; **zero full-caller body evidence** |
| Exact NodeMap::iter body, no added contract | Pass | Pass | Real enumerate/filter_map closure body checks; no cardinality or returned-iterator laws exported |
| Exact NodeMap::item_count calling that iter | Fails item_count | Same | First unsatisfied obligation is native_count_supported(self.iter()), iter.rs:224; not an unsupported-syntax claim |
| Corrected insert/get/set bookkeeping baseline, empty proof slots | Pass | Fails occupied_update | Logical collection facts need one explicit set equality under Infer, despite ordinary success |
| Same baseline + exact specialized add_signature body, no local glue | Fails add_signature | Fails add_signature and occupied_update | Array-value equality and set/cardinality transport remain |
| Final ordinary manual variant, identical executable bodies | Pass | Pass | Both remaining obligations discharged with two erased assertions |
| Separate imported retransmission client | Pass | Pass | Uses public contracts with CLOSED slots body hidden; repeated insertion and frame/count observations check |

Full caller inspection: NodeMap iterates occupied unique slot indices; BTreeMap stores distinct PeerId keys. Keychain's guard is count>=q; engine's guard is count==q and rejects excess counts too. The engine guard supplies **no known-key subset premise**. Runtime membership requires actual verify_schnorr's public-key lookup plus verification. Original Keychain::new/tagged_message/verify, SessionOutcome::header/Merkle/consensus encoding, index conversions and secp256k1 crypto graph have not been checked. secp256k1-0.29.1 verification reaches unsafe secp256k1-sys0.10.1 FFI; no accepted vstd application contract supplies a verifier summary. This is a documented missing checked closure, not an inference from unresolved-import diagnostics that the entire language path is intrinsically impossible.

The actual keychain policy domain has nonempty configured keys (local identity/key validation); adversarial indices and signatures must remain arbitrary. Config validates local key matching and contiguous API endpoints, but inspection is not a proved invariant of every supplied signature map. Decoded NodeMap vectors can differ from configured key count. Do not assume honest input signatures, recognized peers, count equality or iterator correspondence to bypass eligibility. BTreeMap len/iter models require a lawful cmp key; PeerId's full ordering/dependency closure is not proved here. No callback, external_body stub, replacement boolean algorithm, new standard contract/axiom or private iterator/model bridge was introduced.

## Exact replacement guarantee and its limits

`manual.rs` retains original private NodeMap<T>(Vec<Option<T>>) and original public NodeIndex(pub usize). Generic get and insert executable bodies are byte-identical. The original add_signature body is byte-identical after erasing one assertion: `self.insert(index, signature.clone()); self`. The source generic SignatureSet<S> trait implementation is specialized to `[u8;64]` and presented as an inherent method; Self::Signature is resolved to that concrete array. This does **not** certify external trait registration, unused codec/Send/Sync/Debug bounds or full generic Clone behavior. The actual array clone call uses inherited standard clone contracts; array equality is proved, never assumed from Copy. No new clone contract was added.

Domain: ALL slot vectors of arbitrary T for insert/get, every vector of 64-byte signatures for specialized aggregation, every in-bounds usize index. Any original Some/None pattern and arbitrary signature bytes are allowed. Out-of-bounds access panics in original source and is separately rejected by the safety control; an empty vector has no valid insert index. No small federation, fixed length, valid signature or known-peer restriction was imposed.

Authored semantics are exactly three erased definitions: public CLOSED slots = self.0@ inside the original defining type, occupied = finite set of valid usize indices whose slots are Some, and signers = occupied(slots). This is a direct observation of checked concrete state; it asserts nothing about the private iterator. Four authored executable contracts (get/insert/add_signature/observation wrapper) and two proof signatures state the target semantics. There is no assumed cardinality parameter equated to an arbitrary signer set.

New application value: insertion replaces exactly one stored signature, preserves length and all other slots, transforms actual occupied indices by set insertion, and increments actual logical signer cardinality by **zero on duplicates / one on previously empty slots**. The imported client performs three actual add_signature calls at two distinct indices, proves latest stored bytes at each and unchanged values elsewhere. This is independently useful signature aggregation/bookkeeping from the very dependency used by Keychain bootstrap and partial-signature accumulation.

The observation wrapper also calls original checked NumPeers::from/threshold and shows: duplicate insertion cannot cross a logical q threshold; old occupied_count+1<q stays below q after one insertion; one fresh last slot reaches q. Its numeric n is an explicit positive policy parameter, **not yet tied to actual configured public-key collection size**. The import client similarly observes the +2 bound. These are additional logical bookkeeping observations, not actual runtime rejection/admission or the missing concrete-known-peer/quorum library premises. In particular there is no claim that occupied signers are a subset of recognized peers. Original bootstrap_multi/with_size, NodeCount conversions, construction and full Keychain fields remain outside the checked replacement. No guessed conversion, trait-dispatch or private-iterator correspondence is used.

## Literal annotations, inventories and costs

Final incremental mechanical work over preregistered baseline:

- occupied_update: `assert(occupied(s.update(i as int, Some(value))) =~= occupied(s).insert(i));`
- add_signature after original insert: `assert(*signature =~= self.slots()[index.0 as int].unwrap());`

**Two erased assertions; zero new lemma calls, auxiliary helpers, broadcast/reveal/fuel/solver directives, loop invariants or new runtime statements.** A trial clone-identity broadcast was unnecessary and removed. Removing the array equality assertion fails add_signature in BOTH modes. Removing the set equality passes ordinary but fails occupied_update under Infer. No helper inference annotations/candidates exist; all precision checks have an empty candidate set. Inference removes zero authored work and here needs an extra local assertion compared with ordinary checking. No cache-hit/speedup or new common automation contribution is claimed.

Already-authored baseline observation glue is separately charged: one ghost before snapshot, one empty proof block and two semantic assertions (`stored == Some(&signature)` and q==quorum(n)); four exec calls (from/threshold/insert/get). occupied_finite is empty and `Set::finite` is intrinsically true/deprecated: **no application value**. occupied_update has the one assertion above. The independent client has one frame quantifier proof with empty body, three ghost snapshots and nine assertions (including the frame assertion), three add_signature calls and two policy calls; it introduces no trusted facts or helper calls.

Exact final local checked exec identities (crate f3_manual): NodeIndex::clone, NodeMap::get, NodeMap::insert, NodeMap::add_signature (Rust identity NodeMap::<[u8;64]>::add_signature), record_signature. Generic insert's Rust identity is NodeMap::<T>::insert. Checked proof identities: occupied_finite, occupied_update. Three spec definitions are not executable proofs. Backend9/0 includes two generated tuple constructors beyond this scope inventory. Separate imported client checks retransmission only, 1/0; external providers are not double counted in its local scope. Failed crates certify ZERO selected bodies; individually successful dependencies and exact failed identities remain in receipts.

Historical F2 ordinary provider is reconstructed exactly from component.rs + manual.json + adapter.rs and checked/exported ordinarily before use; private artifact/source hashes guarded for imports. Its 8 exec/3 proof scope and quorum laws are inherited accepted evidence, **not new F3 progress**. Neither that rerun nor trivial finiteness counts as this lane's value.

Representative final costs (seconds compiler wall; ms total verification; solver rlimit-run):

| Fixture | Ordinary | Infer |
|---|---|---|
| Corrected baseline | 1.039s /159ms /84743, pass | 2.140s /183ms /113244, fail |
| Empty-glue aggregation | 1.064s /247ms /121075, fail | 2.271s /247ms /149965, fail |
| Final manual audit | 1.322s /227ms /117948, pass | 2.381s /187ms /127944, pass |
| Imported client audit | 1.029s /113ms /58305, pass | 2.485s /102ms /58305, pass |

All retained trials/exploration: **105 compiler launches, 141.910 summed compiler-wall seconds**, including repeated historical F2 provider checks and failed setup/controller epochs. No build waits/builds. These are focused tiny fixtures, not broad validation matrices or performance claims. Authentication, copying, authoring and Python setup are outside compiler time. No blocker was investigated up to the45-active-minute cap: count stopped at its first explicit unsupported precondition, clone proof used five small trial variants then ordinary checked assertions; no prerequisite chain was opened. Git timestamps and command durations are retained; no fabricated exact active-time accounting is claimed.

## Controls and failure accounting

Final selected controls are accepted by the compact controller with exact failed-function/body/helper inventories:

- False unchanged-state contract on original insert, isolated from downstream clients: only insert fails, ordinary/Infer. Example: slot None becomes Some(v).
- False claim add_signature always increments cardinality: only add_signature fails, both modes. Example: insert into an already occupied slot.
- Separate imported duplicate client falsely claims second same-index insertion increases count: only retransmission fails, both modes; original providers remain healthy.
- Remove only insert's in-bounds precondition: only insert fails on original indexed update; retained Seq::update recommendation. This is a SAFETY negative, not a semantic theorem counterexample.
- Individual assertion ablations as above: mechanical tests, not semantic negatives or weakened helper contracts.
- Failed goals-enabled controls suppress interface/export; absent duplicate-infer.vir is then rejected by the real import loader in both modes (`could not open imported library file ... No such file`). Successful ordinary providers/imports are separately guarded and checked. Goals-disabled native --export can leave a .vir after proof failure; this existing behavior is retained/disclosed and such artifacts are never accepted as providers.

Earlier setup/controller epochs are retained separately in RESULTS.json/private receipts: sandbox bwrap loopback failure; first preparation's guessed bitcoin version rejected before compiler; missing rustup PATH; expected-failure mismatch when real iter passed; ill-typed Set adapter; mutable-poststate frontend syntax; expected-failure mismatch when corrected ordinary baseline passed; distinction between ordinary success and Infer failure; source-only missing crate graph; initial client missing dependent native library search path. Some first broad semantic/safety controls caused downstream contract failures/recommendations and were superseded by isolated insert-only controls. Initial recorder confused encountered-error with type/frontend errors (proof failures set it too); final validation uses precise errors/diagnostics, full scope and exact failed identities. None of these is silently counted as semantic evidence. No solver resource exhaustion was observed.

## Parent handoff / next sequents

1. Full Keychain: first concrete unresolved sequent is native_count_supported(returned NodeMap::iter), before further iterator-law/bounded-decrease/count-to-occupied-set obligations; then actual verify/index/crypto closure. The iter body alone is already supported. Do not manufacture a correspondence or replace native .count().
2. Full engine: provide a genuinely checked runtime header/keychain/verification closure and PeerId ordering/dependency context; guard cardinality alone cannot establish recognized signer subset. Whole-source setup probes do not identify a native theorem or discharge this closure.
3. Replacement: all frozen F3-B slot/aggregation/observation sequents check ordinarily, plus imported client. Full generic trait/bootstrap construction and policy-configuration linkage remain unselected/unproved; no automatic follow-on foundation is authorized.
4. Measured remaining mechanical demand was set-update equality plus array clone extensionality, not elementary Vec frame transport. No sibling reads and **NO demonstrated shared REDB obligation**; propose no common engine/vstd work. Parent should obtain the required NEW independent review and decide whether this bounded bookkeeping value warrants any further scope.

The previous practical milestone remains completed historical input. Original HMAC/native remains **DEFERRED AND UNACHIEVED**, native correspondences zero; old44+3/101 and HMAC suffix are untouched.

Replay: `python3 -B -O source/tools/research/practical_caller_transitions/fedimint/prepare.py /tmp/<fresh-private-root>` authenticates/copies inputs without trials. Pass that root via --private to replay.py, always with fresh --output. Scenarios eligibility/count/corrected-baseline/aggregation/manual cover checkpoints; semantic/safety/duplicate/omit-array/omit-set cover controls. client/client-negative take --provider pointing at a successful manual output; failed-import also takes --rejected pointing at the absent failed Infer export. RESULTS.json links every selected/exploratory private receipt with hashes, inventories and costs; FINAL_FILES.json pins final sources/controller. No Python assert is used as a validation gate; optimized Python runs retain explicit checks.
