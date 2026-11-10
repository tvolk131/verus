#!/bin/bash
set -eu
source /home/dev/.t3/worktrees/verus/t3-3d5de66f/source/tools/research/stbor_checked_foundation/private-env.sh
export OPAMKEEPLOGS=true
opam_bin=/tmp/stbor-checked-foundation-20261110/downloads/opam-2.3.0-x86_64-linux
# Serial packages bound total build concurrency to two jobs.
for package in ocamlfind.1.8.1 num.1.3 camlp5.7.08 coq.8.9.1 \
  coq-stdpp.dev.2019-11-22.2.16eb222f coq-iris.dev.2019-12-20.0.62798412 \
  coq-paco.4.0.0 coq-equations.1.2.1+8.9; do
  "$opam_bin" install -y --jobs=2 "$package"
done
