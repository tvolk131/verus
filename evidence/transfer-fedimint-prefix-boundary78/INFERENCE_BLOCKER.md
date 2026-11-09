# Reusable Infer blocker: original prefix Vec append

Fresh initial probe and both runner probes use the original three executable
bodies, native Vec/slice/Range syntax, original record fields and `--infer-goals
--no-cheating` with default erasure. No contract or projected body is supplied
to Infer. They request three functions and generate ZERO.

Typed result: `status="unsupported"`, `crate_success=false`,
`encountered-vir-error=true`, `interface_written=false`.

Exact `generation_failure`:

```text
Verus Infer unsupported: calls require an acyclic same-crate inferred callee or certified imported summary: Fun(Path(alloc, ["vec" :: "impl&%3" :: "extend_from_slice"]))
```

The native `full_key.extend_from_slice(key)` call is the observed first blocker.
Ordinary Verus checks the same three bodies with authored contracts using the
existing vstd library specifications. This is useful source-connected verification
but NOT inference progress. The early return/order of this diagnostic does not
establish Infer support for Vec clone, generic cloning or the entire range body.
An unrelated exploratory reversed-concatenation control using `slice::to_vec`
was also frontend-unsupported; it is preserved and never counted as semantic
evidence. The final reversed control uses supported Vec new/extend operations.

## Bounded future proposal, NOT implemented or approved

1. Recognize exact standard Vec/slice clone/extend calls with checked, explicit
   element-clone semantics (start with u8), allocator/type/mode checks and
   old/final-state view contracts. An ordinary library specification must not
   silently become a certified inferred application's effect summary.
2. Carry the cloned vector and mutable append state through the existing effect
   representation without replacing record fields or native executable bodies.
   Check borrowed slice nonmutation, append order and full final vector view.
3. Check generated contracts with the ordinary backend, no cheating/default
   erasure; reject unsupported clone relations, alias/effect patterns and trait
   dispatch fail-closed. Do not infer namespace incomparability from integer IDs.
4. Bind exact source/native identities and library dependencies to candidate
   identity77/policy71/schema38-compatible publication machinery (or an explicitly
   reviewed new identity), with local/imported positives, order/suffix mutants,
   explicit safety/frontend categories and seeded failed-export removal.

No production engine work, new application axiom, unsupported native equivalence,
or desired-conclusion external body is part of this cycle. Stop at this bounded
pure-key bridge; async dispatch, backend honoring ranges/keys, token escape,
serialization and transaction commit/drop remain separate refinement obligations.
