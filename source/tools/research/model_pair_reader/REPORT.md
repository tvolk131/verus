# Checked constructive pair reader milestone

Private frozen trial 010 normally compiled all nine declarations, inventoried
all assumptions, and passed full recursive coqchk without exclusions. Receipt
and object identities are in constructive-milestone.json. The first three-law
milestone was committed as ab8471ae and fully checked in trial 002.

The constructive specialization of Share uses existing resolve_uniq_body,
consumes the internally constructed unique key, retains its internal 0/0 indices,
and constructs genuine shared ty_own at d=1,g=0. It executes the real Share
expression; it does not postulate a typed rule or bound the old existential.
The exact original shared tctx is also constructed at synchronized depth 1/1.

raw_pair_reader_context_wp takes only initial raw fragments, time context and
2+k credits; it internally allocates the same-Outer TypeG, two lifetimes, both
model references, and the actual shared reader. One credit initializes the
lifetime context, one prepares the reader, k remain for the callback. The public
raw_pair_reader_observed supplies 2+k inside model adequacy, with no original
application premise for credits, owners, points-to resources or guards.

The callback retains padRef/Lpad, exact outside frame and exact pair shared tctx;
it receives persistent fixed Cap/key_reader, linear R/G, the G lifetime guard and
a closure joining R/G to ordinary Lkey. Each indexed read costs one credit. The
parent may use R * loopcredits as Rin, keep G outside and join afterwards; no
parent Rin/Rout interface was edited.

whole_key_heap_observe preserves the same genuine heap_ctx sigma, ordinary Lkey,
Cap and arbitrary Frame using (g+1)*keylen explicitly reserved credits, reporting
exact original key values with arbitrary existential RSt r. It extracts no
exclusive vector. The prefix induction has a no-access zero case. Integration
controls for empty aliases, same-allocation interiors and an actual first read
are being added.

MODEL only. Original Rust bodies/domains untouched; actual native=0; all 44
minimal, 3 phase, 101 effects open; full HMAC 1/1 inherited unchanged;
publication=false; goal_complete=false. No new axioms/admissions/primitives.
