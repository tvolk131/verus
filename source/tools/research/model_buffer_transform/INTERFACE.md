# Checked memory loop for a Zip prefix and its untouched suffix

Prove a reusable operational heap-language loop law: read a pad cell and key
cell, execute a separately checked binary operation, write exactly that pad
cell, then advance. For n no larger than either input length, return the entire
pad transformed in its first n positions, its original suffix, and the entire
unchanged key. Framing retains every surrounding resource. The full writer
specializes n to min(pad length,key length), including empty and oversized keys.

The binary operation's WP is an explicit checked proof obligation, not an
assumed XOR summary. Do not claim the XOR instance until that operation is
implemented and proved. Use actual existing heap reads/stores and exact write
credits. No language primitive or operational semantics is added. This generic
memory law supports later original-source simulation; the recursive model
expression is not a replacement accepted Rust fixture or a proof of native Zip.

Freeze candidates before trials; normally compile and recursively kernel-check
complete assumptions. Preserve all original bodies/domains and keep native
correspondence/publication false until the original source loan, native
operations and C/F observations are connected. No caller permission premise is
to be added to the original writer. Internal WP resources describe the model
composition and may later be supplied by the checked heap initializer.
