# Reviewed declaration inventory

All names below were read in the exact candidate source at review base
91eee431; generated register principles are additionally printed from the fresh
object. Every declaration has an empty individual assumption inventory. Full
printed bodies/types and normal-compile/kernel logs remain in private outputs
bound by the review receipt. No Program definitions or obligations occur.

## NativeRegisterSegment.segment: 30 declarations

Candidate: `source/tools/research/native_register_segment/candidates/segment.v`.

| Declaration | Exact role / hypotheses / conclusion |
| --- | --- |
| registers | `gmap nat descriptor`; pure descriptor map, no cell resource. |
| instruction | One datatype constructor, not arbitrary native instructions. |
| AssignPure | Destination nat, operation, ordered source-register names. |
| register_arguments | `Forall2` of exact successful source-register lookups and descriptors. |
| register_step | Prop relation from instruction/registers/state to registers/state. |
| RegisterCompute | Requires register_arguments AND existing `source_step op ds σ [out] τ`; updates only destination with out. |
| segment_run | Prop relation for a finite list of these instructions. |
| SegmentNil | Empty code leaves map and state identical. |
| SegmentCons | Threads the actual intermediate map and state between one step and remaining code. |
| register_step_heap | ANY admitted register_step has τ=σ, by inversion of existing source_step. |
| register_step_other | For AssignPure dst, any name≠dst preserves its lookup. |
| segment_run_heap | ANY admitted segment_run preserves complete physical state. |
| end_segment | Exactly AssignPure 6 UnwrapThin [3]; AssignPure 5 ByteOffset [6;2]. |
| end_registers | Insert RawThin p at 6, then RawThin(shifted p n) at 5 in the original map. |
| end_segment_exec | Input 3=NonNullThin p, 2=Usize n, offset_valid p n imply the two-step authored register run to end_registers and unchanged σ. |
| end_segment_outputs | Both exact output lookups, at 6 and 5, for end_registers. |
| end_segment_frame | Every name other than 5 and 6 retains its original lookup. |
| end_code | One existing-lrust function; lexical r6 binds actual identity/projection result and feeds Offset with len. |
| chain_pure | PureExec True n a b and PureExec True m b c compose to n+m. |
| end_code_four_steps | ANY p,n: complete App end_code reduces in four deterministic pure steps to pointer_loc(shifted p n). No finite guard on the target-only lemma. |
| wp_end_code | ANY p,n,R and lrustGS: R entails WP of complete App, exact returned location and R. |
| end_segment_refinement | Same two lookup premises and offset_valid; R entails pure register execution AND WP of complete target with exact register5/result relation and R once. |
| instruction_rect | Generated dependent elimination of instruction into Type, with exactly the AssignPure case. |
| instruction_ind | Generated dependent elimination of instruction into Prop, with exactly the AssignPure case. |
| instruction_rec | Generated dependent elimination of instruction into Set, with exactly the AssignPure case. |
| instruction_sind | Generated dependent elimination of instruction into SProp, with exactly the AssignPure case. |
| register_step_ind | Generated Prop induction on RegisterCompute; retains argument-lookup/source_step premises, no new semantic case. |
| register_step_sind | Generated SProp elimination on RegisterCompute; same premise/case coverage. |
| segment_run_ind | Generated Prop induction: SegmentNil base, SegmentCons with register_step, remaining run and its induction hypothesis. |
| segment_run_sind | Generated SProp induction with those same two cases. |

There are 15 definitions/lemmas/theorems, 3 inductive names, 4 constructors and
8 generated eliminators. Kernel constant inventory is 23 = 15+8; the three
inductive checks include their four constructors. No missing `register_step_rect`
or `segment_run_rec` is invented: these Prop relations have only the generated
`ind`/`sind` principles in their compiled namespace.

## NativeByteXor.native_xor: 26 constants

Candidate: `source/tools/research/native_byte_xor/candidates/native_xor.v`.

| Declaration | Exact role / hypotheses / conclusion |
| --- | --- |
| bits | Width-bounded little-endian list via Z.odd and Z.div2, all Z inputs. |
| word_xor | Structural pairwise Boolean xorb, truncates to shorter list. |
| word_value | Positional sum using 0/1, addition and multiplication by 2. |
| scalar_xor8 | word_value(word_xor(bits 8 a)(bits 8 b)); definition has no Z.lxor. |
| bits_length | length(bits n z)=n for all n,z. |
| word_xor_length | Length is min(length xs,length ys). |
| word_value_range | For every Boolean list xs, 0≤value<2^(length xs). |
| bits_value | Under unsigned n-bit range, positional decoding of bits n a equals a. |
| odd_lxor | Low bit of literal Z.lxor equals Boolean xorb of low bits, all integers. |
| div2_lxor | div2 distributes over literal Z.lxor, all integers. |
| word_xor_bits | Boolean XOR of n decoded bits equals n decoded bits of literal Z.lxor. |
| scalar_xor8_lxor | byte_value a AND byte_value b imply scalar_xor8 a b=Z.lxor a b. |
| scalar_xor8_range | byte_value(scalar_xor8 a b) for ALL integers a,b. |
| scalar_xor8_pure | Under both byte bounds, concrete xor_call reduces by PureExec True 3 to literal scalar_xor8. |
| scalar_xor8_normalizes | Under both byte bounds, for ANY fixed σ, exactly 3 actual primitive steps, no observations/forks, same σ. |
| cell_assignment | Existing SC read at p; call actual byte_xor on loaded a and supplied b; existing SC write at SAME p. |
| register_file | Ordinary function nat→option Z, not linear/native register state. |
| without19 | Maps exactly 19 to None; every other n to original rs n. |
| prepare_assignment | If rs19=Some b, return (cell_assignment p b,without19 rs); otherwise None. |
| prepare_assignment_some | Successful 19 lookup implies that exact pair. |
| prepare_assignment_none | Missing 19 lookup implies rejection. |
| moved19_unavailable | Returned without19 map has None at 19. |
| other_register_preserved | Any n≠19 preserves rs n. |
| wp_scalar_xor8 | Both byte bounds, lrustGS: WP actual xor_call with exact scalar_xor8 literal result. |
| wp_cell_assignment | Both byte bounds; old p↦a * £1 * arbitrary R; WP actual read/call/write returns poison, p↦scalar_xor8, same R, result byte range. |
| wp_prepared_assignment | Successful 19 lookup and both byte bounds; exists expression/updated map satisfying prepare_assignment, 19 absent/all others equal, AND the same conditional cell WP with old ownership/£1/R. |

Exactly 8 definitions/fixpoints and 18 lemmas. Zero datatypes, constructors,
generated eliminators, nested modules or Program obligations. Assumptions and
ProofTerms inventory modules each contain exactly 52 trivial marker theorems,
all included in the recursive check; these are not extra candidate statements.
