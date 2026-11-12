# Launcher-only preparation failure

The first probe's PATH omitted /home/dev/.cargo/bin, so the accepted verus launcher could not find rustup. It emitted no verifier JSON and never reached source translation. The controller retained the failure, then its JSON parser rejected the empty output. /tmp/redb-crash-backend-nominal-20261112-v1/receipt.json and ordinary.stderr preserve this setup failure. It is not an unsupported-source result or semantic control.

The second prospective freeze changes only the launcher PATH and adds an explicit --freeze argument. The generated fixture/source/type declarations are unchanged. No additional nominal type or behavioral fact is introduced.
