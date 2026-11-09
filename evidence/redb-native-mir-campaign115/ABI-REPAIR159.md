# Parent-staged F159-1 correction: source-only, no native credit

Source input: immutable158 primitive158.rs SHA256
1bedb36cbb36c6d9ddb96451b4fb60950c4424ab6d25ad77ba7cad3a8addbd8a.
Staged copy: /private/tmp/redb-parent-abi-repair159-20261109T0012Z/primitive158.rs
SHA256 b0e62e8ee6254fc060ee4c3fca6c5b47f81d0a1c6548e506a2c85f2bb12c7117.
Retained ABI-REPAIR159.patch SHA256
6370eaba9b596ff307fddf99e344cf434eb185df6eea0beecef9e37c21759af1.

The first allocator operand remains checked usize. The second must be the
actual selected runtime Layout argument's align-field type, with Layout bound
to the compiler alloc_layout lang item, and the nominal pinned core Alignment
ADT retained. No nominal-to-usize equality or repr transmute is assumed.
The separately required numeric Alignment::as_usize correspondence stays OPEN.
The patch is uncompiled; typechecking/API compatibility and actual applicability
must be validated within160's existing allowance, not by a new repair delegate.

No old source was edited. The original158 source still matches its pinned hash.
No compiler/build/verifier/solver/native process ran in this parent repair.
The initial combined shell command stopped after writing the patch because zsh
reserves the variable status; copying/hashing completed in a separate command.
This failure is retained, not retrofitted into a worker receipt or allowance.
