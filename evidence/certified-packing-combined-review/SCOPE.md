# Independent combined review — scope

Review base: 4d5ce2b86ed5a176d327f9bdcd13b5c3a08b178b; production: 21eeabbb6e0eade862798e4dbe022d69c4a564c5.

This review checks actual foreign MIR/VIR correspondence and proof-produced publication/import evidence together with the exact proofless full Item generator (SHA256 1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d). It requires a fresh offline locked Rust 1.99.1 jobs2 compiler/macros/vstd build, immutable matching provider/consumer compiler bytes, full-domain proofs and ordinary import visibility. It tests all-query and erased-compilation publication gates, pruning/transitive evidence, cold/warm precision proofs, runtime/body inventories and focused regressions.

All new tracked material stays in this directory. Raw sources, logs, binaries and complete receipts stay in a new private /tmp directory. No production changes, push, PR, subdelegation, live parent/sibling reads, race/substitution/security experiments, new premises or disabled-check positives. Historical evidence is provenance only. Python gates use exceptions and must pass with -O -B.

No native theorem or whole-goal completion is claimed: native correspondences=0; 44 minimal +3 phase /101 writer effects OPEN; inherited canonical6/0 and HMAC1/1 UNPROVED are not fresh results. Bounded certified .vir publication is distinct from global publication=false and goal_complete=false.
