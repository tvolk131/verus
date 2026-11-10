# Existing model XOR loop with a genuine shared key

Prove the existing, unchanged ModelBufferTransform.zip_transform applied to the
existing ModelByteXor.byte_xor using ModelSharedByteRead.session.key_reader.
Use the actual indexed SC-read law and its internally issued linear R token.
The key is never supplied as an exclusive physical vector. Keep arbitrary Frame
and shared aliases. No new loop algorithm, language primitive or reader axiom.

The core invariant should track original key base/index while the unchanged loop
advances its pointer. For arbitrary start/count, require count<=length pad and
start+count<=length key. Return exactly the transformed prefix with unchanged pad
suffix, R, key capability and Frame. Read budget is g+1 per step and store budget
is one per step, so an explicit count*(g+2) budget is a natural sufficient bound.
Preparation is outside this law. Derive actual accessed key byte/range from the
checked reader; every key lookup must match the original key vector.

Specialize start=0 and count=min(length pad,length key), including arbitrary
initialized 128-byte pads and every key length, including empty and >128. State
exact XOR prefix, untouched pad suffix and byte range. Root separately constructs
the shared view/reader internally and composes the changing pad loan and final
heap observer. Native Rust Zip correspondence remains unproved.

Only this new directory is owned. Freeze candidates before execution, normally
compile, inventory every constant/obligation and recursively kernel-check with
exact dependency source/object/receipt pins. No parent/sibling checkout reads,
baseline/vstd/compiler edits, axioms, admissions, unsafe flags, new application
premises or fixture/domain changes. Model WP is not native execution or total
termination. Native correspondences remain zero and publication/completion false.
