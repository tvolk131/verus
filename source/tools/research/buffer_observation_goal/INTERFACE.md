# Checked native-for observation interface (implementation lane)

`native_mutable_for::describe(Function, function_map)` returns a bounded
`NativeMutableFor { loop_expr, buffer, capacity, key, wrapper, iterator }`.
It recognizes resolved core IterMut and Zip<IterMut<u8>, Iter<u8>>, with
closed catalog signatures and native identities. Initial capacity budget is
exactly 10 or 128; immutable key slice length is unrestricted. Unsupported
shapes skip, with no executable rewrite. This interface is internal VIR, not
a trusted summary or new vstd contract.

`buffer_observation_goals::prepare(functions, imported, report)` is the checked
annotation interface for composition. Run numeric annotation first, then this
pass; its full-function projection restores exactly the incoming numeric pass
output. The observation pass proposes two ghost Seq snapshots before borrowing,
length facts and pointwise current/final referent/view equalities. These are
ordinary checked declarations/invariants. It supplies no index-to-application
counter alignment, arithmetic law, body semantics or unyielded-tail frame.
The semantic prefix invariant remains authored and unchanged. Parent/numeric
passes may consume the descriptor and later share checked annotation proposals;
recognition reuse alone is not a shared-automation result.

Status: compiled and ordinary-checked prefix milestone; see REPORT.md and
manifest.json for exact outcomes and limits. Full frozen canonical and HMAC
targets, bodies and domains remain unchanged. Prefix success is not transfer
completion. Plain/canonical observations were redundant in standalone controls.
