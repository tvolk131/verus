# Campaign91 operational stop and final-review gate

The first native feasibility pass found a concrete obstruction to the proposed
unchanged, full-u32 operational/safety theorem. Native construction at
`new(u32::MAX, u32::MAX)` followed by `clear(u32::MAX - 1)` produces a state
satisfying the candidate storage, hierarchy and padding invariant. With overflow
checks enabled, native `find_first_unset` and `alloc` panic when the search
computes the last word's exclusive endpoint as `entry + 64`.

Research evidence: `/private/tmp/verus-operational91-research.9BZD1V/REPORT.md`.
The exact12-body closure, candidate invariant, native token correspondence,
ordinary executable witness and12/16 verifier launches are retained there.
This is not a complete operational reference proof. The48 reference launches
remain unused, and no conditional Infer implementation cycle is triggered.

Independent review must establish source fidelity, invariant validity and the
checked-overflow counterexample before accepting this diagnostic. It must also
distinguish an internal full-u32 counterexample from supported application
reachability: actual page/region callers have smaller limits, and release
wrapping behavior is not the checked-arithmetic safety theorem. No production
exploit, supported-database crash or release-mode failure is claimed.

Do not silently add a size precondition, alter native arithmetic, substitute a
target or treat the helper proofs as the required allocator theorem. Those
would require a new scope decision. The campaign therefore stops operational
proof/feature work at PARTIAL rather than spending the remaining proof budget.

Consolidation and measurement are implemented separately. The new source-built
runtime has fresh matched-vstd and three qualified native replays; final review
must authenticate them and the docs/shared-tooling changes. The measured
preflight-hashing recommendation remains unimplemented: no speedup is claimed,
and no extra optimization or feature cycle is started after the operational
stop. Small final-review repairs remain bounded by the original two-round cap.

The historical blocked goal remains unfinished; goal API refusal does not
authorize falsely completing it. Final handoff must say CLOSED_PARTIAL unless
the original required operational theorem actually exists.
