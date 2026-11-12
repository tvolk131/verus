# Retained pretrial setup refusal

The first freeze-generation attempt looked for rlibs under Cargo's output target directory; preparation explicitly uses a separate private build directory. It refused before writing FAULT_FREEZE.json. A dependent replay was mistakenly invoked after that failed command; /tmp/redb-crash-native-20261112-faults1/receipt.json records a missing-freeze refusal. No compiler or native process ran. The corrected freeze pins the unique rlib under prepare2/build/release/deps before any trial. This setup failure is not a semantic negative or native result.
