# Registration correction before trials

The initial source-registration script expected Cargo.lock and stopped with FileNotFoundError before writing CASES.json or replacing WORK_STATE. The three charter files were committed separately at d91be0cf; no proof/native trial ran. The completed caller-state archive was already copied and is preserved byte-identically.

This pinned REDB library/workspace has no Cargo.lock. CASES.json pins the actual source and records the absent lock instead of inventing an upstream transitive identity. A native preparation, if needed, must register a private exact dependency graph/lock from available cached published sources before builds or fault trials and disclose that it is a prospective native context, not an upstream lock. No network, shared cache update or dependency program is authorized by this correction. Existing verifier-source probes need no native Cargo graph.
