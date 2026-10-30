#!/usr/bin/env python3
"""Exercise syntax-inventory visibility; these are parser checks, not proofs."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile


SOURCE = r'''#![allow(dead_code)]
mod external;
macro_rules! hidden { () => { fn generated() {} } }
hidden!();
#[cfg(any())]
mod disabled { fn still_in_source() {} }
mod inner {
verus! {
    struct Counter { value: u8 }
    impl Counter {
        pub open spec fn view(&self) -> int { self.value as int }
        pub fn step(&mut self, n: u8)
            requires n < 255,
            ensures self.value == n,
        {
            proof { assert(n == n); }
            fn local_helper() { }
            while false invariant true decreases 0 { }
            self.value = n;
        }
        unknown_impl_macro!();
    }
    trait Model {
        fn observe(&self) -> int;
        unknown_trait_macro!();
    }
    #[verifier::external_body]
    fn opaque() -> u8 { 0 }
    proof fn unsound() { assume(false); }
    spec fn hinted() -> bool { forall|i: int| #[trigger] predicate(i) }
    pub assume_specification [foreign] (n: u8) -> (r: u8) ensures r == n;
}
}
#[cfg(test)]
mod tests { fn test_only() { assert!(true); custom!(opaque_tokens); } }
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    args = parser.parse_args()
    binary = args.binary.resolve()
    with tempfile.TemporaryDirectory(prefix="adoption-inventory-check-") as directory:
        root = Path(directory)
        path = root / "sample.rs"
        path.write_text(SOURCE)
        command = [str(binary), str(root), "sample.rs"]
        run = subprocess.run(command, capture_output=True, text=True)
        assert run.returncode == 0, run.stderr
        report = json.loads(run.stdout)
        assert report["schema_version"] == 1
        entries = report["entries"]
        by_name = {entry.get("name"): entry for entry in entries if entry["kind"] == "function"}
        assert by_name["step"]["owner"] == "inner::Counter"
        assert "requires" in by_name["step"]["authored_signature_spec"]
        assert "ensures" in by_name["step"]["authored_signature_spec"]
        assert by_name["step"]["loops"] == 1
        assert by_name["view"]["mode"] == "spec"
        assert by_name["unsound"]["mode"] == "proof"
        assert by_name["test_only"]["test"]
        assert by_name["still_in_source"]["attributes"]
        assert any("external_body" in attr for attr in by_name["opaque"]["attributes"])
        assert any("allow" in attr for attr in by_name["step"]["attributes"])
        kinds = {entry["kind"] for entry in entries}
        assert {"external_module", "opaque_item_macro", "opaque_impl_macro",
                "opaque_trait_macro", "assumed_specification"} <= kinds
        findings = {finding["kind"] for entry in entries for finding in entry.get("findings", [])}
        assert {"proof_block", "loop_annotation", "assume", "manual_trigger",
                "native_assert_macro", "macro_call", "nested_function"} <= findings
        custom = [finding for entry in entries for finding in entry.get("findings", [])
                  if finding["kind"] == "macro_call" and finding["path"] == "custom"]
        assert custom and "opaque_tokens" in custom[0]["tokens"]
        # A malformed source file must fail instead of producing an empty audit.
        path.write_text("fn broken(")
        bad = subprocess.run(command, capture_output=True, text=True)
        assert bad.returncode != 0 and not bad.stdout.strip()
        empty = subprocess.run([str(binary), str(root)], capture_output=True, text=True)
        assert empty.returncode != 0 and not empty.stdout.strip()
    print("Source inventory exposes authored clauses, proof aids, assumptions, opaque macros and cfg-disabled syntax; malformed input fails")


if __name__ == "__main__":
    main()
