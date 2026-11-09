# Parent structural repair163

Sealed follow-on to independently reviewed, disabled continuation160. This is
not a retry or completion of160, and does not consume its remaining allowance.
November 8, 2026 America/Chicago; November 9 UTC.

## Changes and demonstrated scope

- Loan destinations now refuse any overlapping live parent/child holder rather
  than checking only the exact destination key.
- Complete structural regression programs reproduce both invalid overlaps in
  the baseline checker, then reject with the corrected checker. A disjoint loan
  destination remains a positive control.
- Store/allocation selection retains separate bounded call paths instead of
  globally deduplicating by Instance. Recursive paths refuse. These selectors
  compile; no original-MIR ambiguity execution or semantic injectivity proof
  is claimed.
- An existing unrun coverage fixture now explicitly exits both terminal blocks,
  so it reaches the intended coverage refusal rather than ResourceLeak.

Only four candidate source files differ from sealed160. No parent compiler,
original application source, native source, or sealed160 input was modified.

## All six charged commands

1. baseline-loan-regressions: exit101, 24 passed/3 failed. Two initial regression
   fixtures and the existing coverage fixture prematurely reached ResourceLeak.
   This is retained fixture failure, not the complete overlap counterexample.
2. complete-loan-counterexamples: exit101, 25 passed/2 failed. Complete programs
   use/end the loans, consume resources, and exit. The baseline checker returned
   success for both invalid overlap cases. This is an executable structural
   counterexample, not a native execution or SMT SAT result.
3. corrected-structural-tests: exit0, 27 passed/0 failed/204 filtered. Exact
   recorded source entry and exit agree with the sealed corrected sources.
4. corrected-compiler-build: monitor stat raced a transient compiler object;
   FileNotFoundError retained. The owned build was killed/reaped with exit-9.
5. corrected-compiler-build-monitor-fixed: exit101. The repaired monitor no
   longer fails on disappearing transient files; the compiler build instead
   refused unavailable Git version metadata. No Git identity was fabricated.
6. corrected-compiler-build-snapshot: exit0. The existing supported snapshot
   build interface received the exact explicit toolchain and canonical sorted
   source-map SHA256. Final compiler build and its source entry/exit agree.
   Snapshot identity is not Git provenance or bootstrap correspondence.

All six direct children were reaped; all six recorded process-group identifiers
were absent at the November 9, 2026 00:54 UTC parent check. Recorded resource
samples: minimum free20,782,907,392 bytes, peak allocated797,843,456 bytes, peak
logical788,342,709 bytes. These are sampled bounds, not continuous observations.
One exploratory search of a nonexistent rustc_tools_util path also returned2;
that discovery error is not an additional compiler invocation.

## Custody and limitations

Raw streams, exact argv/environment, clocks, source entry/exit maps and failures
are retained for every command. The original initial-test fixture is reconstructed
from retained edit deltas and checked against its recorded entry digest; that
reconstruction is not an independently preserved historical source snapshot.
Final source copies, baseline-relative patch, packaging script and binary hash
are sealed. No continuous custody, transitive toolchain authentication,
reproducible bootstrap, original native acceptance, or independent repair review
is claimed. Build warnings remain in raw stderr.

F161-1's structural counterexamples are repaired/tested. F161-2's path collapse
is removed and compilation succeeds; native applicability remains unobserved.
F161-3/F161-4 remain the substantive implementation164 target: interpreted
Alignment value, native dynamic event/full-pointer correspondence, initialized
affine resource continuation and global actual control-flow/resource effects.
No outputs become initialized authority in this repair.

All18 OPEN; one conditional mathematical allocator schema; operative native
laws0, issuance0, accepted original allocations/stores/bodies0/0/0. Native export
REFUSED. Historical16019/32 and15817/32 resource failures stay sealed unchanged.
The heavy slot is released only after this completed build and current sealing.
