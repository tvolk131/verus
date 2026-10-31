# Original unsafe public vector endpoint phase

Baseline native cc0bf36056a190bd7da4cbf226e778580e8bb472;
engine 1e0cf81d20d0dcec0f137edc9b95a8d9c2d98d60;
iterator acceptance archive 0b512160f. Prior accepted source/evidence is immutable.

Required behavior for arbitrary Rust str's original UTF-8 bytes B:
odd byte length returns OddLengthString containing B.len before invalid-byte
inspection. Even byte length succeeds iff every byte is an ASCII hex digit.
Success has length B.len/2 and element i = 16*digit(B[2i])+digit(B[2i+1]).
Otherwise return InvalidChar with the lowest invalid byte index and that exact
original byte, retaining native high/low priority. Empty input succeeds empty.
No ASCII-input precondition or allocation success/invalid-data shortcut is added.
Rust allocation failure/divergence remains the standard execution model.

Select the original public decode_to_vec, error conversions and drain_to_vec
behind proof-only cfg verus_hex_vec_phase3. Check required postconditions on the
original native source, not a copied/safe adapter. Keep ordinary v2 replay usable.
No first-party assume, external body or excluded native statement may bridge a
blocker. A failed probe publishes no interface.

Memory acceptance additionally requires real with_capacity and as_mut_ptr
allocation identity/capacity, pointer provenance and in-bounds write/add
operations, exactly one initialized byte per consumed successful pair, and
set_len only after the entire prefix is initialized. Early Err must preserve
Vec's actual zero length and justify dropping allocation with a partially
written u8 prefix. Native len/size_hint used for allocation must be checked,
not imported because of an ExactSizeIterator name. Reuse the checked pair and
forward transition theorems, and add general permission/initialization support
and unrelated regressions if practical. Safe replacement or assumed drain
postconditions do not certify this endpoint.

If this requires substantial new general ownership/frontend support, retain a
reproducible opt-in failed native proof with independent requirements, exact
failed source/logs/artifact hashes and a concrete next-phase support boundary.
Do not call it an accepted vector theorem. Ordinary restored v2 proof and native
feature/MSRV tests must pass before the phase work is committed locally.
