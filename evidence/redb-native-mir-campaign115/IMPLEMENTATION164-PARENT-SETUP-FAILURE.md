# Parent intake164 initial setup failure

The first authentication script exited1 after all1508 archive/current-member
checks succeeded. It incorrectly interpreted final-source164.json paths as
relative to candidate/source, rather than candidate. The exact failure was an
AssertionError at the inventory path-set comparison. Diagnostic comparison
showed candidate1120/1120 paths, extras0/missing0; candidate/source897 paths,
with expected source/ prefixes absent. No payload mutation was identified.

authenticate.initial.py retains the original script. The corrected script
uses candidate as the inventory root. No compiler, application or verifier
execution occurred. This setup failure does not reset any historical allowance.
