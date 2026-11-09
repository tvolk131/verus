# Native writable-page caller premises — November 7, 2026

**PARENT SOURCE INSPECTION / UNREVIEWED / NO NEW TRUST OR PROOF CREDIT.**
This trace narrows the concrete gap between the bounded primitive gate and
read_header's real cache/I/O caller. It is not a new contract, a checked MIR
trace, a native execution, a discovered native bug, or authorization to implement
resource semantics. All 18 database/history obligations remain OPEN.

## Exact source scope

Recovered redb pin: `c82ab865bb362bc4af067d332adc44978ff64813`.
Root: `/private/tmp/verus-crash-consistency94-20261105/native-leaf-lookup-007/recovered-native/redb`.
The five inspected files match inherited022 `source-inputs.json` content hashes:

| File under src | SHA256 |
|---|---|
| tree_store/page_store/cached_file.rs | cb901d44531b23f87d1bd31f72a24f1dc69308b5417db2167b7508921ffda486 |
| tree_store/page_store/lru_cache.rs | a5896391195b2da0b94b14d1641fb2916528fb520406214162544e8253dcdcd4 |
| tree_store/page_store/base.rs | ce2ae1188b5bad2babe2f7b7df19d82adc2b8baff6b41b957e978437686938c4 |
| tree_store/page_store/page_manager.rs | b062608a527c3f53aa505af870f2b803da3fa7b095169194c681cc6441cb558a |
| sync.rs | 933159b9b6b93500df1d517f961fe8c07c3fc07ed54aa2527a0f95a4ffea5016 |

Inherited source-input map SHA256:
`ae1a01cfed4013038bca9eab85b2b20b537aa7efef69360da5aa1190117cb530`.
Content binding restores neither original native paths/Git continuity nor
compiler/runtime/type correspondence. The map and selected files are unchanged.

## Checkout does not establish uniqueness merely by removing a cache entry

`cached_file.rs:24` has TWO different Arc roles: buffer owns the cache-container
handle, while data owns the byte-allocation handle. Their control blocks and
frames must remain distinct even though both use Arc syntax.

- write at881 locks the offset's write-buffer stripe, removes a read-cache entry
  if present, and first tries take_value at95 on the write cache. take_value
  moves its Some payload out and leaves an existing None reservation.
- On a cache miss, write chooses the removed read-cache Arc, zero_filled_arc,
  or read_direct_into_arc. It inserts Some into the write cache and takes it
  back out, leaving None. The result pairs that data with stripe.clone() and
  the actual offset. The clone is of the cache-container Arc, not the byte Arc.
- zero_filled_arc at19 uses repeat_n/collect; read_direct_into_arc at643 mutates
  that fresh result with get_mut before a backend read. Their actual constructor
  and library/resource closure are not supplied by A1–A3 alone.
- read at651 returns cloned data handles from cache hits, including the write
  buffer. A removed cache entry therefore does not, on its own, prove that all
  previously returned handles have been retired. Cache locks serialize access
  to their maps, not the complete lifetime of a returned Arc.

Consequently the successful mutable caller needs an established history/lease
invariant: the selected data is the intended allocation, complete strong/Weak
inventories satisfy the success domain, no incompatible live byte loan exists,
and outside handles cannot violate that condition. Native transaction/page
discipline may establish it, but this local source inspection does not. No
publicly reachable alias violation or panic is inferred. In particular, do not
infer global no-alias from &mut WritablePage or from the stripe mutex alone.

Reservation ownership must separately bind the SAME cache identity and offset:
exactly one outstanding checkout corresponds to the None entry; unrelated
removal, invalidation, eviction, shrink or flush cannot silently consume it.
The native eviction path skips checked-out None entries. Shrink at558 explicitly
expects no outstanding writable pages. These checks require caller/state proofs,
not an assumption that every hash-map entry is always Some.

## Return uses a clone, then generated destruction retires the old field

WritablePage::drop at40 calls lock/unwrap and passes self.data.clone() to
return_value at91. return_value replaces the existing option and asserts that
its old value was None. For the intended unique-data successful history, the
resource argument must cover this sequence, not merely a borrowed-u64 Drop:

| Phase | Required resource relationship, NOT a checked result here |
|---|---|
| Before Drop body | Original data handle h, live C/A, and the matching cache/offset None reservation; all incompatible byte loans already ended. |
| Clone and return_value | Fresh data handle e refers to the SAME C/A and bytes; e is moved into the matching cache entry, which changes None to Some(e). Original field h still exists. |
| After user Drop body | The mutex guard has its real checked end; cache owns e. Returning from this body alone has not proved field destruction. |
| Generated field glue | Retire the original data field h once while cache handle e remains. Also account separately for buffer's Arc and its guard/allocator/glue closure. |
| Before following successful flush | No outstanding None reservation at any visited entry; cache-owned data is the exact modified allocation and bytes. |

The bounded A3 worksheet preserves an original h and retires its freshly created
extra e. Native return instead keeps e in the cache and later retires the old
field h. The current gate does not itself establish this handoff. A future proof
must supply checked typed role/ownership transport and actual nonfinal glue,
or seek review of an explicitly different extension. Swapping display names or
assuming generic Drop is not that proof. This is an application/scope gap, not a
counterexample to the proposed bounded A3 law.

The actual compiled evaluation order, temporary guard end, generated field-drop
ordering, partial initialization and exceptional edges remain E2/E4/E5 checks.
Poisoned-lock unwrap, missing reservation or an unexpected Some select panic
behavior whose terminal/runtime closure is still unresolved; no successful
return/frame follows from those paths. sync.rs selects std versus spin locks
by cfg, so an unbound lock spelling establishes neither progress nor poisoning.
PageMut at base.rs:256 has its own debug-only Drop at280 plus nested field glue;
this does not collapse into the bare WritablePage header-write case.

## Why this matters to real interrupted recovery

page_manager.rs:879 read_header flushes successfully before clearing
primary_unflushed when required, then copies serialized bytes into a temporary
WritablePage and calls storage.flush afterward. A native bridge must check that
the temporary's actual successful return/glue puts the exact modified page back
in the correct cache BEFORE that following flush visits its stripe. Replacing
this edge by a byte-slice adapter or an assumed cache-return contract would hide
the required theorem.

flush_write_buffer at490 unwraps every visited cache value before writing it;
successful closure must exclude live None reservations. On a write error the
first loop may already have issued writes, so no rollback is implied. Its later
loop moves data into the read cache or drops it according to the budget; that
is another ownership transition, not perpetual preservation of original h.
Native backend sync, partial I/O, physical length, flag clearing and higher
history/KV invariants remain separate obligations from any successful cache
handoff proof.

## Concrete next action, still subject to authorization

Use checkout/reservation/data-handoff/glue as the first real caller proof
boundary for any authorized resource-engine candidate. Before admitting the
caller, establish the constructor and alias-history premises, account for the
new surviving cache handle versus old field retirement, and preserve the actual
error/cleanup successors. Passing the existing 17 primitive fixtures alone
cannot close this boundary. No additional primitive law, scoped admission,
implementation/build allowance, proof replay or fixture run is granted here.
