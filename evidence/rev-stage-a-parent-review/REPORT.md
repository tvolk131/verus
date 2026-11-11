# Independent review of provisional parent f4415999

Recommendation: retain/accept **inactive StageA only**, subject to the parent's broad pipeline and acceptance. No blocking production defect was found in the reviewed dormant slice. This does not promote the accepted engine (d3b2aa6e, policy74/cache77; docs1171bdf7). Reviewed parent is schema39/policy75/cache78. StageB and activation are not reviewed or accepted.

The implementation is a compiler-TCB extension: `Internal` supplies ordinary nominal datatype encoding across modules while source access remains private. It is not a native-refinement theorem. There is no source installer, rev_iter definition, aggregate native helper, native reverse loop, new standard axiom or independent future equality. Native0/HMAC1/1/44+3/101 remain OPEN; publication=false; goal_complete=false.

## Review findings and limits

* The source/backend query split is consistent across the enumerated callsites. Old Never/WhenVisible branches retain their previous visibility/encoding semantics; `is_visible_to` delegates to the same previous owner query. Prune, Ctx invariant propagation, native/poly boxing, datatype/SST AIR, resolvedness and field reach follow encoding. Source WF, error expansion, inference observation privacy and constructor registration retain source gates. Constructor expressions, field expressions/places, patterns and `shr_ref_struct_wrap` converge on the source gate; public contracts require suitably public field access. Constructor function items explicitly exclude Internal. Record/newtype/wrapper inference admission continues to require the older allowed modes. No source producer was found.
* A single nominal constructor/selector is used regardless of module. Generic fields are substituted through existing datatype machinery; boxed values use the same datatype box/unbox identity. The existing ordinary `borrow_mut_to_sst` obtains current(field), creates its normal reference, and emits phase2 assigning future(reference) to the original field place. SST `update_field` reconstructs the same nominal datatype; mutable-reference update preserves the outer future. The existing synthetic test checks current, writeback and unchanged outer future; removing writeback yields Invalid, not an unsupported/panic result. Independent controls also check nested nominal fields and the inherited u8 invariant, including a false <=254 bound. Ordinary recursive/boxed source smoke is separate from Internal.
* The factory's private descriptor members prevent source/import construction, but the factory and hash tuple are compiler code/TCB. The raw tuple is neither a cryptographic certificate nor a native correspondence. Metadata checks pin stable core crate/ADT/field identities and actual loaded private-field/generic shape. Descriptor validation rereads the actual descriptor-bearing VIR object at WF and Ctx entry. Field type must be precisely the single TypParam: decorations, boxed/ref forms and wrong types are refused. `types_equal` generally ignores ImplPaths, but its exact TypParam comparison here cannot admit a decorated or datatype field. Independent poly tests use pointer identity to confirm the full argument object, including a nonempty ImplPaths and decoration, survives coercion.
* The diagnostic driver loads the **actual** Rev<Enumerate<slice::Iter<genuine Item>>> metadata. Its VIR datatype is manually assembled, not the result of a production translator/installer. This limits the driver to metadata plus factory diagnostics. It is not evidence that native-to-VIR source binding, future observation definitions, constructor MIR routing or native bodies have been established. Likewise, source smoke uses an ordinary visible private struct; Internal remains reachable only from compiler/test IR.
* The serde wire claim holds for the actual enum. The independent test uses actual serde_json and bincode-next3.1.1 with the production legacy configuration. Old Never=0 and WhenVisible=1 encode identically to the old two-variant enum and roundtrip both directions (public and private visibility). Constructed Internal and its enclosing datatype refuse encoding. Unknown/truncated tags and JSON Internal forms refuse decoding. Tags0..255 were also boundedly checked: no decoded value is Internal. The derive implementation omits Internal from its deserialize cases; this structural restriction, rather than finite payload enumeration, supports the universal non-construction claim. This is compatibility of these existing enum values, not a general cross-version VIR-format promise.
* Schema39 is unchanged. Actual fresh ordinary Bool provider/import works. Typed old-schema38 and old-policy74 trailers and a truncated ordinary base fail closed before proof. The early Internal export branch inspects the selected pre-crate before pruning with no helper dependency and calls the existing all-destinations invalidator. Existing library tests exercise removal of both destinations and continuing after one removal fails. No source-reachable Internal producer exists, so the complete activated export/cache/refusal matrix has **not** been exercised or claimed.

## Independent verification

All proofs use this review's fresh matching compiler, builtin, macros and full normal vstd, with Rust1.99.1, explicit RUSTUP_HOME=/home/dev/.rustup, offline/locked Cargo, jobs2 and private home/cache/target/intermediates/tmp. No historical executable bundle supplies proof tools. Exact manifests, core/compiler/target/tool hashes, command receipts and inventories are bound by RESULTS.json and INVENTORY.json; raw outputs remain in the private root recorded there.

| Check | Result | Meaning |
|---|---:|---|
| VIR library suite | 157/0, zero ignored | Includes existing eight StageA synthetic tests |
| Additional review IR/wire tests | See RESULTS.json | Test-only private source copy, actual WF/Ctx/poly/SST/AIR, not source activation |
| rust_verify library | 8/0 | Includes existing invalidation and structural controls |
| Existing constructor tests | 13/0 | Existing-source constructor/privacy/mode compatibility |
| Full NORMAL vstd | 2250/0 | No no-verify or filtered-library verification |
| Exact original Item | 132/0 | 16 checked exec bodies, zero proof bodies, one existing helper |
| Wrapper | 28/0 | Five generated helpers, existing normal compatibility fixture |
| Packet | 31/0 | Zero generated helpers |
| Bool provider/import | 2/0; 1/0 | Normal certificate publication/import compatibility |
| Bool false contract | 0/1 | Ordinary semantic failure; no VIR error |
| Ordinary private-field reborrow | 3/0 | Zero helpers, normal source/body/lifetime/trait/mode/erasure checks |
| Generic/nested/recursive source | 4/0 | Ordinary source semantics, separate from Internal |
| Actual core metadata | accept Rev; refuse Enumerate and changed descriptor | Diagnostics, not native body proofs |
| Native Rev field/source attribute | E0616; unrecognized attribute | Rust privacy and unsupported forged source attribute diagnostics |
| Private pattern/constructor/public spec | refuse | Existing source privacy/public-contract gates |

App commands use --no-cheating, --compile and whole-crate verification with normal lifetime/trait/body/mode/erasure checks. Runtime MIR inventories for Item and ordinary reborrow were independently emitted by pinned rustc with ghost erasure; they inventory runtime bodies, not verified native standard bodies. Body/helper/trusted-model inventories and AIR query counts are retained separately, without conflating queries, functions and runtime bodies.

Excluded attempts are retained and classified in RESULTS.json: default Cargo doctests after a passing VIR library subset; missing test-harness release layout and constructor include fixture; ambiguous dependency selection; an unrelated input guard catching an edited diagnostic test during another command; a diagnostic test compile typo; and wrong expected counts/JSON default assumptions. None is counted as a semantic negative. Final accepted checks are not pooled with those attempts. Parent's broad722 compiler/Python148/canonical26/observer23 pipeline is not this review's claim.

## Remaining StageB gates

Before activation, bind the actual loaded rev_iter definition, declaration/attributes and dependencies; install exact source/type lowering; validate actual constructor/source-call routing and whole helper Functions before/after normal WF/autospec/mode processing; verify fresh ordinary helper bodies; exercise the full activated local-only export/cache/fingerprint/refusal matrix, including producer/core/target/semantic binding mismatches; and independently establish the intended native identity/lawfulness/minimal-loop result. No lawfulRev/selector/source-native claim or new import/publication authority follows here.

The old f90a6679 checkpoint and worker records are preserved. Worker hashes were rebound through this checkout's own Git38be1482, not sibling paths. The parent's formatter correction uses the actual Rust2024 edition; no formatter-induced semantic delta was found. Only this new review evidence directory is committed. Parent owns production fixes and acceptance.

Execution exception: the initial `find .. -name AGENTS.md` search was interrupted after producing no output; it may have traversed sibling directory entries. No sibling file contents were opened by that search or used as inputs. Subsequent inspection and all builds/proofs used only this checkout and the explicitly allowed completed inputs.
