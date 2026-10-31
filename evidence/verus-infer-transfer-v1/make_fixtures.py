import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def main():
    original = (ROOT / "original-selected.rs").read_text()
    prefix = """#![feature(proc_macro_hygiene)]
use vstd::prelude::*;

verus! {
global size_of usize == 8;

pub open spec fn radix_power(exponent: nat) -> nat
    decreases exponent,
{
    if exponent == 0 { 1 } else { 128 * radix_power((exponent - 1) as nat) }
}

pub open spec fn canonical(number: u64, encoded: Seq<u8>) -> bool {
    1 <= encoded.len() <= 10
    && (number as nat) < radix_power(encoded.len())
    && (encoded.len() > 1 ==> (number as nat) >= radix_power((encoded.len() - 1) as nat))
    && forall|index: int| 0 <= index < encoded.len() ==>
        #[trigger] encoded[index] as nat ==
            (number as nat / radix_power(index as nat)) % 128
            + if index + 1 < encoded.len() { 128nat } else { 0nat }
}
}

"""
    authored = "#[verus_spec(encoded => ensures canonical(number, encoded@))]\n"
    ordinary = prefix + original.replace("pub fn u64(", authored + "pub fn u64(", 1)
    inferred = prefix + original.replace("pub fn u64(", "#[verus_infer]\npub fn u64(", 1)
    inferred += "\n" + authored + """pub fn encode_policy(number: u64, buf: &mut [u8; U64_LEN]) -> &[u8] {
    u64(number, buf)
}
"""
    fixtures = ROOT / "fixtures"
    fixtures.mkdir(exist_ok=True)
    for name, source in (("ordinary.rs", ordinary), ("inferred.rs", inferred)):
        path = fixtures / name
        if path.exists():
            raise SystemExit("Refusing to overwrite fixture " + name)
        path.write_text(source)
    inputs = {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
              for path in sorted(fixtures.glob("*.rs"))}
    inputs["original-selected.rs"] = hashlib.sha256(original.encode()).hexdigest()
    (ROOT / "pre-proof-fixtures.json").write_text(json.dumps(inputs, indent=2) + "\n")
    print(json.dumps(inputs, indent=2))


if __name__ == "__main__":
    main()
