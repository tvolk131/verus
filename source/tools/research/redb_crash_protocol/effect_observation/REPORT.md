# Confirmed native-state observation boundary

The six-case accepted diagnostic completes at `/tmp/redb-crash-effect-observation-20261112-v2/receipt.json`. Ordinary and Infer give the same results:

| Diagnostic | Result | Meaning |
| --- | --- | --- |
| Native AtomicBool new/store/load, no value postcondition |1/0 | Calls, memory-order preconditions and body are supported. |
| Exactly the same body, requiring its final read to be true |0/1, intended postcondition only | Current native atomic contracts do not express stored-value evolution. This is not evidence that the expected Rust property is false. |
| Native Mutex<u64> lock/unwrap/deref | Frontend rejection | Native Mutex, MutexGuard, PoisonError and Mutex::lock lack admitted specifications in this context. Not a semantic negative or an application proof. |

All cases use accepted78 unchanged, schema39/policy78/cache81, private complete runtime copies/caches, --no-cheating and ordinary checks. Inferred helpers are explicitly empty. These are new diagnostic bodies, with **zero extracted REDB executable bodies certified** by this experiment. No type proxy, assume_specification, view law, native-state bridge, engine/vstd change, disabled check or new production fact was added.

The exact source connection is narrower than the requested theorem: original CheckedBackend::check_failure/sync_data use native io_failed/closed atomics. Original PagedCachedFile::flush_write_buffer iterates Vec<Arc<Mutex<LRUWriteCache>>> stripes and moves buffers into RwLock-protected read caches. Under the selected std build, src/sync.rs reexports std::sync primitives. The accepted source atomic contract, byte-identical to5ecea5fe, expressly provides no support for reasoning about internal atomic values. Its load/store specifications constrain valid memory orders without relating returned/stored state. Compiler suggestions to add native lock type/function specifications are not installed.

Thus the15/0 backend body result cannot establish a latch-value theorem or the needed cache/write-trace correspondence merely by supplying more arithmetic/loop annotations. Nominal type admission would not itself establish native guard-to-cache value identity, release/update framing or backend persistence. Actual checked lock/cache/source interpretations remain unproved; it is not justified to replace them with trusted summaries or assume the desired successful-flush coverage conclusion.

The first trial stopped on a diagnostic-text mismatch after four expected atomic outcomes and the ordinary lock rejection; CLASSIFIER_CORRECTION.md retains that refused receipt. The second freeze changes the text matcher only. RESULTS.json records both raw receipt/log hashes and compiler times. No timeout, panic or resource failure was accepted. The accepted matrix has two body proofs, two missing-value-relation failures and two frontend failures, all with explicit scope classification; it does not add six proof results.

The application objective remains unachieved, and no new foundation work was begun. HMAC/native remains DEFERRED AND UNACHIEVED. A changed execution scope would be needed before experimenting with new native state/representation semantics; GOAL.md currently forbids automatically starting that chain.
