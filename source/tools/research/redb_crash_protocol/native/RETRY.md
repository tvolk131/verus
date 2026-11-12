# Native smoke adapter correction before execution

The first frozen offline preparation at /tmp/redb-crash-native-20261112-prepare1/ generated the private lock and compiled unchanged REDB/libc, then failed to compile the harness: get_owned returns OwnedAccessGuard, whose decoded bytes are obtained through value(). No smoke/fault executable ran. This is an API-adapter setup failure, not a crash-safety counterexample or a changed REDB algorithm.

Before the retry, add only the decoded-value binding in check_pair and an explicit --freeze option to the unchanged preparation workflow. FREEZE_2.json binds those edits, all original/native inputs, and the complete first failed receipt and logs. The actual data/profile/property remain identical. Use a fresh private output so the rejected graph, harness and diagnostics stay unchanged. No additional dependency, new I/O law or fault schedule is introduced.
