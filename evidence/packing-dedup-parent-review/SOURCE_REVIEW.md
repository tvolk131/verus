# Source review at prepared 5d79f711

Reviewed against accepted engine 21eeabbb; reviewed module SHA256
`d384bdd6091b89a83612529e6dc9bfba1573dd7c35018941f6befacb3b536f72`.
No actionable source finding identified. Runtime validation is reported separately.

## Identity argument

`ClosedKey(KeyTyp, KeyExp)` includes the type of every visited expression.
`KeyTyp` is injective on the accepted undecorated Bool, mathematical Int, and
U8/U16/U32/U64 subset. All other types retain their checks. `Constant::Int`
includes its entire BigInt. `VarIdent` derives Eq/Hash over both textual identifier
and the complete disambiguator enum (including every enum payload). Only bound
references with matching types are keyable; no ambient variable can enter a key.
`BinaryOp` derives structural Eq/Hash and includes its enum payload: inequality,
arithmetic operation, bitwise operation, shift width and signedness flag. Both
children occur in order. No commutation or simplification occurs.

`BndX::Quant` has precisely quantifier, ordered VarBinders, ordered nested Trigs,
and AssertByLocals. The last must be None. `Quant` has only the quant field; it
must be forall. Each binder's full name and type enters the ordered vector.
Duplicate names, zero/more-than-three binders, unsupported binder types, nested
quantifiers, and exhausted 4096-node traversal return None. Trigger group order,
expression order, expression types, binder order and body are all preserved.
Thus, by structural induction, key equality implies identical supported closed
typed syntax modulo diagnostic expression/binder spans. The supported syntax
contains no function/model/type-resolution payload left out of this argument.
`SpannedTyped` consists of span, type and expression; `Spanned` adds only span.
The spans in these generated formulas supply locations and occurrence labels,
not premises, operators, binders or types. AIR occurrence-label normalization
used in the evidence is strictly an inventory comparison, never the selector.

The key does not promise well-formedness: e.g. malformed binary operand/result
sort combinations can still be keyable. Their first exact copy is retained for
ordinary validation. Removing a later identical copy cannot turn that malformed
formula into a valid one. Unsupported/free proposals always retain every pair.
A false proposal can only be dropped if an earlier identical false proposal
remains to fail. The three new production units inspect true/distinct-false/free/
unsupported retention, disambiguators, types/triggers/operators, first original
Arc, empty requires, immediate proof/use pairing, and a fresh second scope.

## Scope, order, original construction

The HashSet is created inside append_checked, once per prepare invocation; no
static, export, dependency, cache or cross-function table is consulted. HashSet
requires full Eq even after a hash collision. Iteration is over the unchanged
proposed vector, never the set. The first exact original Arc survives and every
retained AssertBitVector has empty requires and exactly that original Arc in
ensures, immediately followed by Assume of the same Arc. Both duplicate
statements are removed together; the original function body follows unchanged.
The exact byte comparison in authenticate/final audit covers all original
number/constant folding/discovery/formula/bounds code and unchanged emitter body.
There is no new helper/library/model trust or application premise.

## Publication and compatibility

Outside this module the only production changes against 21eeabbb are
INFER_POLICY v71->v72 and candidate registry v74->v75; schema remains 39.
The cache stores proposals, not proofs; a hit still requires fresh precision
proofs. Global ordinary, inference, auxiliary error counters and deferred errors
remain unchanged. finish_verus requires actual successful ordinary body queries
for every local foreign helper. publish_verified_interface checks global errors;
the driver calls it only after the final erased Rust invocation succeeds.

Imports reject incompatible schema, policy or producer before returning facts.
Foreign certificates are compared to the actual consuming compiler's typed core
identity; summaries and fingerprints bind evidence and transitive dependencies.
Pruning a private helper cannot remove its required foreign certificate. No
historical compiler certificate is used as a positive import in this review.
