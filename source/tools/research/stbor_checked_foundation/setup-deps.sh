#!/bin/bash
set -eu
source /home/dev/.t3/worktrees/verus/t3-3d5de66f/source/tools/research/stbor_checked_foundation/private-env.sh
opam_bin=/tmp/stbor-checked-foundation-20261110/downloads/opam-2.3.0-x86_64-linux
"$opam_bin" repository add --all-switches coq-released /tmp/stbor-checked-foundation-20261110/repos/coq-2020/released -y
"$opam_bin" repository add --all-switches iris-pinned /tmp/stbor-checked-foundation-20261110/repos/iris-2020 -y
"$opam_bin" install -y --download-only --jobs=2 --json=/tmp/stbor-checked-foundation-20261110/manifests/download-plan.json \
  coq.8.9.1 coq-iris.dev.2019-12-20.0.62798412 \
  coq-stdpp.dev.2019-11-22.2.16eb222f coq-paco.4.0.0 coq-equations.1.2.1+8.9 \
  camlp5.7.08 num.1.3 ocamlfind.1.8.1 conf-findutils.1
