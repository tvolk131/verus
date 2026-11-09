# Source-only branch-to-leaf routing assessment

## Verdict

NEW feasible conditional slice: the native branch-selected slot is the only
supplied child leaf containing the query; its lookup equals lookup over their
union, with exact associated values. This is an AUTHORED composition client,
not new native accessor credit, native get/fetch/ownership, or policy adoption.
Branch023's eleven immutable accessors/partition are ALREADY proved and reviewed.
No new primitive trust: child ranges are explicit caller premises, not axioms
or established native invariants. Unconditional native descent remains blocked.

## Roots, pin, evidence

Abbreviations used below and in the adjacent input-hash table:

* E = /Users/tommyvolk/.t3/worktrees/verus/t3-60978068/evidence
* L = /private/tmp/verus-crash-consistency94-20261105/native-leaf-lookup-007
* B = /private/tmp/verus-crash-consistency94-20261105/native-branch-routing-023
* R = /private/tmp/verus-routing023-independent-review-5nRzcqsG
* N = L/recovered-native/redb/src

Native target pin remains c82ab865bb362bc4af067d332adc44978ff64813.
N/tree_store/btree_base.rs hashes to the requested
c3a1f5ca0cefdde0bc7d1b40536bc4f2d68a8bce852115433621a8b37fb9ec60.
This is RECOVERED-CONTENT binding, not Git ancestry: the parent HEAD observed was
b32846b61dcf724618f7345e219e5007a0c79643; read-only git cat-file/show could not
resolve the target native commit here. No checkout, fetch, Git write, or repair.
Read E/redb-crash-consistency94/{GOAL.md,STATUS.json,TAKEOVER.md}; all18 remain
OPEN. Partition100/admission101 are separate context, not dependencies or rechecked.

Bounded novelty search: campaign reports, branch023 source/clients, R/REPORT.md
and review-callers.rs, evidence references and nearby routing reports.
B/REPORT.md:44 proves partition/exact page; B/clients-003.rs:102 and
R/review-callers.rs:12 prove routing uniqueness. R's new callers add accessor
agreement/numeric examples/structural extrema, not child-leaf union lookup.
No such composition found in inspected inputs; NOT exhaustive absence.
Nearby twolevel reports are bitmap growth. No forbidden task was resumed.

## Original immutable executable anchors and domains

N/tree_store/btree_base.rs:

* 1825: BranchAccessor<'b,T:Page>, fields page, num_keys, fixed_key_size.
* 1832 new: debug tag check, reads u16 LE count from bytes[2..4].
* 1860 total_length: key_end(num_keys-1); not safe at zero count.
* 1865 child_for_key: midpoint binary search comparing QUERY to separator;
  Less -> max=mid, Equal -> return (mid, child_page(mid)), Greater -> min=mid+1.
  Equality belongs LEFT of the separator. Final return uses min_child slot.
* 1887 key_section_start; 1896 key_offset; 1904 key_end; 1920 key;
  1929 count_children; 1933 child_checksum; 1946 child_page; 1960 num_keys.
* 517 LeafAccessor::new; 544 position; 568 find_key: actual leaf search chain.

N/types.rs:231 total-order declaration; :237 separator left<=s<right; :251
default returns left; :983 le_impl!/:988 comparator/:997 u32 use decoded scalar,
NOT lexicographic LE. Arbitrary user implementations are not thereby proved lawful.

Checked B/branch-003.rs:82 fields, :114 new, :135 child_for_key require :36
branch_bytes_wf and sorted_for<K>, valid query. Counts0..65535, children1..65536,
fixed/variable widths (structurally Some(0)), bounded monotone endpoints, spare
tails, length<=usize::MAX. Metadata8+24*(n+1)+(variable?4*n:0)<=1835012.
:106 post: 0<=c<=n, separators before c<query, at/after c>=query, exact
decoded_child(bytes,c), canonical fields NOT allocation/file-fit. key Some iff
i<n; page/checksum Some iff i<=n; total_length needs n>0; key_end needs valid i.

L/leaf.rs:456 Value valid/meaning/number; :467 Key comparison is integer-meaning
Ordering; :479 u32 valid iff length4, meaning=LE decode; :493 checked compare.
Reflexivity/order follows from that contract/implementation, NOT sortedness.
L/consumer.rs:39 canonical_u32 proves semantic equality iff byte equality for
four-byte u32 using LE lemmas, NOT arbitrary Key injectivity. :48 search_u32
requires ordered_leaf/query length4; gives presence iff encoded key exists and
exact associated value. L/leaf.rs:54 requires1..65535 pairs, endpoints<=u32::MAX,
length<=usize::MAX. No empty-leaf/corrupt-byte guarantee.

Native N/tree_store/btree.rs:1087 get_helper; :1094 leaf; :1097 Arc/AccessGuard;
:1101 branch; :1106 get_page/error; :1108 depth error. None is the authored client.

## Precise new theorem and executable client

Use arbitrary four-byte u32 keys/value bytes within existing domains, both width
modes. Native u32 fixed_width is Some(4); None is only a byte-accessor domain.

Define m(x)=leaf_decoder's u32 meaning. Add a CHECKED proof lemma for len(x)=4:
m(x)==<u32 as leaf_lookup023::Value>::meaning(x). Both open definitions reduce
to the SAME vstd LE decode. Distinct crates/traits are not silently identified.

Let S_i be B/encoded_key(branch,kw,i), n=key_count(branch). For child j define
interval(j,x) := (j==0 || m(S_(j-1))<m(x)) &&
                 (j==n || m(x)<=m(S_j)).
Separators are upper-inclusive for their left child, lower-exclusive for right.

Authored signature sketch, not original redb code:
route_then_search<'a>(branch:&[u8], leaves:&'a [&'a [u8]],
 kw:Option<usize>, vw:Option<usize>, query:&[u8])
 -> (usize, pagecodec023::PageNumber, Option<(&'a [u8],&'a [u8])>).

Preconditions (all explicit, no assumed desired result):
1. branch_clients023::ordered_bytes::<u32>(branch@,kw), query.len()==4.
2. leaves.len()==n+1; every leaves[j] satisfies lookup_client::ordered_leaf
   with the SAME kw and vw; every key in leaves[j] satisfies interval(j,key).
No precondition says the query exists, that the returned value is correct, or
that the chosen index already satisfies the desired cross-child conclusion.

Executable body is only:
let (c,p)=branch_clients023::route_u32(branch,kw,query);
let hit=lookup_client::search_u32(leaves[c],kw,vw,query);
return (c,p,hit), with checked ghost reasoning between/after calls.

Postconditions:
* c<=n; p==branch_routing023::decoded_child(branch@,c), canonical fields only.
* query is in interval(c,query); every matching key in ANY supplied leaf j has
  j==c (quantify j and key index, preserving their bounds).
* hit.is_some() iff exists j,k: encoded_key(leaves[j],kw,vw,k)==query@.
* Some((key,value)) has key@==query@ and exists j,k with that key AND the exact
  returned value@==encoded_value(leaves[j],kw,vw,k).

Proof implementation: instantiate routed_bytes at c-1 and c for query interval.
For any matching key in leaf j<c, instantiate route at separator j: S_j<query,
but leaf upper bound says query<=S_j. For j>c, use separator j-1: query<=S_(j-1),
but leaf lower bound says S_(j-1)<query. Both contradictions use the checked
cross-crate meaning bridge and byte-equality substitution. Thus j=c. Invoke
search_u32's existing iff/exact-value result; move existential witnesses between
selected leaf and union. Use explicit triggers/witnesses; no new loop/tree model.


## Existing imports: concrete no-build solution

Do NOT assume leaf_decoder exports BranchAccessor or that leaf_lookup023 is the
same nominal crate. Keep both namespaces, prove their u32-meaning bridge above.
Use retained RELEASE artifact pairs, never B/artifacts (overwritten with debug
generation). For each pair pass --extern name=rlib and --import name=vir:

* leaf_lookup023: B/runs/attempt-001/{libleaf_lookup023.rlib,leaf_lookup023.vir}
* pagecodec023: B/runs/attempt-002/{libpagecodec023.rlib,pagecodec023.vir}
* branch_routing023: B/runs/attempt-019/{libbranch_routing023.rlib,branch_routing023.vir}
* branch_clients023: B/runs/attempt-020/{libbranch_clients023.rlib,branch_clients023.vir}
* leaf_decoder: L/runs/attempt-009/libleaf_decoder.rlib + L/leaf-final.vir
* builder_step: L/runs/attempt-010/libbuilder_step.rlib + L/builder-final.vir
* lookup_client: L/runs/attempt-011/liblookup_client.rlib + L/client-final.vir

builder_step is transitive, not invoked. Release invocations019/020 historical
B/artifacts input hashes MATCH retained outputs001/002/019; complete-scope
receipts. L/REPLAY.md:21 identifies leaf bundle. No re-export/overwrite.

Runtime is L/midpoint-runtime-proposal/runtime/verus; hash
da0aad5ec3359b3037f522f3153570242a65529d8696b2b7209fd48599046628,
matching B/runtime/verus. Exact nine-file runtime comparison and all pair hashes
are in the input table. Sysroot is
/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin.
TCB: matching vstd/std slice/array/StructuralEq, six LE bindings, midpoint,
compiler/solver/sysroot. No new external_body/assume/admit, type/comparator laws,
page validity, resource semantics or policy. No fresh full closure certification.

## One bounded verification-only plan (NOT executed here)

One worker, <=16 apps including failures, <=15min wall, <=128MiB NEW scratch.
No builds, --compile/--export, network or parent/Git edits. Reference runtime,
never copy it. 128MiB denotes scratch; if RSS was intended, feasibility is
unestablished: stop. Free disk>16GiB each launch; per-app wall75s/log caps.

Use original011 invocation's environment (offline, 1.99.1, DYLD libraries,
VERUS_Z3_PATH, VERUS_IN_VARGO=1), with ALL writable HOME/TMP/cache/output paths
redirected into fresh scratch; never run the sealed runners in place. Command:
runtime/verus --sysroot SYSROOT --edition=2024 --crate-type=lib
--crate-name branch_leaf_client --no-cheating --output-json --num-threads 1
--triggers-mode silent --multiple-errors 12 --rlimit 30
-C debug-assertions=no [seven exact --extern/--import pairs and dependency -L]
client.rs. No function filters or skipped proof body. This is verification only.

Applications1..4: import/meaning bridge probe; generic full composition client;
nonvacuous fixed-width byte witnesses; variable-width counterpart witnesses.
Keep final client plus witnesses in each subsequent variant's full local scope.
For witnesses build AUTHORED borrowed byte arrays (no native producer claim):
branch n=2, separators256 and65536, metadata80, total88 at fixed4, page-word
slots at56/64/72 with raw1/2/3; checksums occupy8..56. Three leaves each have two
pairs: [1,256], [257,65536], [65537,u32::MAX]. Fixed4/fixed1 leaves have14 bytes
with values at12/13; take SIX arbitrary u8 values. Query1/256/257/65536/65537/max
tests exact key/value transport, query0/2 absence. Numeric1 vs256 is nonlexical.
For variable mode branch metadata88 and keys through96; leaf variable key/value
endpoint tables must be explicitly checked, not assumed. Query is arbitrary in
the generic theorem; concrete calls are satisfiability witnesses, not its scope.

Apps5..6: matched semantic negatives, not equality-route reproof:
(a) same lawful pages/query256, native route unchanged, deliberately search
leaf1 instead of leaf0 and assert the global-presence equivalence; one intended
assertion fails because leaf1 lacks256 while leaf0 contains it;
(b) use lawful generic client, query257, claim returned value is leaf0's value
rather than leaf1's corresponding arbitrary byte; one intended assertion fails.
Keep corrected counterparts/native bodies unchanged. App7 restores full positive;
8..16 are shared corrections, not promised success. Retain all inputs/argv/raw
outputs/scope/diagnostics. Negatives require ONE intended backend failure, no
precondition/safety/frontend/resource/import errors. Stop on incompatibility or
exhaustion: no provider rebuild/trust workaround.

## Smallest native dependency still missing

Smallest missing checked native relation: a reachable branch's decoded child
slot fetches THAT child image, whose descendant keys obey adjacent separator
intervals under the same lawful comparator. Sortedness/canonical fields do not
establish this. Native producer/tree-invariant establishment/preservation and
fetch-image identity must supply it; PageImpl/Arc/AccessGuard/errors remain open.
No axiomatic replacement, get_helper promise,18-cell promotion or publication.

## Assessment execution record

Started2026-11-07 19:03 America/Chicago. Zero verifier/solver/compiler/build/test/
native-target/network/subdelegation/parent edits. Only report/input hashes written.
Disk>16GiB. WALL LIMIT EXCEEDED:8m12s by19:11:13, before final trimming.
