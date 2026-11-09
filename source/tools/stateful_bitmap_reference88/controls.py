"""Frozen, independently selected bridge and ordinary-import negative obligations."""

import hashlib
import json
from pathlib import Path
import sys

BRIDGE_CASES = {
    'zero_divisor': ('precondition not satisfied', '''
fn zero(numerator: u32) -> u32 { numerator.div_ceil(0) }
'''),
    'missing_divisor_premise': ('precondition not satisfied', '''
fn missing(numerator: u32, divisor: u32) -> u32 { numerator.div_ceil(divisor) }
'''),
    'false_ceiling': ('assertion failed', '''
fn false_goal() {
    let value = u32::MAX.div_ceil(2);
    assert(value == 0);
}
'''),
    'false_floor': ('postcondition not satisfied', '''
fn floor(numerator: u32, divisor: u32) -> (result: u32)
    requires divisor != 0,
    ensures result == numerator / divisor,
{ numerator.div_ceil(divisor) }
'''),
    'wrong_type_u64': ('div_ceil` is not supported', '''
fn wrong_type(numerator: u64, divisor: u64) -> u64 { numerator.div_ceil(divisor) }
'''),
    'wrong_method': ('checked_div` is not supported', '''
fn wrong_method(numerator: u32, divisor: u32) -> Option<u32> { numerator.checked_div(divisor) }
'''),
    'wrong_identity': ('postcondition not satisfied', '''
struct Other { value: u32 }
impl Other {
    fn div_ceil(self, divisor: u32) -> (result: u32)
        ensures result == self.value,
    { self.value }
}
fn identity(numerator: u32, divisor: u32) -> (result: u32)
    requires divisor != 0,
    ensures result as int == (numerator as int + divisor as int - 1) / divisor as int,
{ Other { value: numerator }.div_ceil(divisor) }
'''),
    'wrong_method_ceiling': ('postcondition not satisfied', '''
fn wrong_method(numerator: u32, divisor: u32) -> (result: Option<u32>)
    requires divisor != 0,
    ensures result == Some(((numerator as int + divisor as int - 1) / divisor as int) as u32),
{ numerator.checked_div(divisor) }
'''),
}

IMPORT_CASES = {
    'growth_false_flag': ('assertion failed', '''
fn wrong(data: Vec<u64>, length: u32)
    requires 0 < length,
{
    let result = resize_read(data, 0, length, true, 0);
    assert(!result.1);
}
'''),
    'false_length': ('assertion failed', '''
fn wrong(data: Vec<u64>) {
    let result = resize_read(data, 0, 1, false, 0);
    assert(result.2 == 0);
}
'''),
    'false_padding': ('assertion failed', '''
fn wrong() {
    let result = resize_read(Vec::new(), 0, 1, true, 0);
    assert(!live(result.0@, 63));
}
'''),
    'false_frame': ('assertion failed', '''
fn wrong(data: Vec<u64>)
    requires data@.len() > 0, live(data@, 63),
{
    let result = resize_read(data, 0, 1, false, 0);
    assert(!live(result.0@, 63));
}
'''),
    'missing_representation': ('precondition not satisfied', '''
fn wrong(data: Vec<u64>, old_len: u32) {
    resize_read(data, old_len, 1, false, 0);
}
'''),
    'invalid_access': ('precondition not satisfied', '''
fn wrong(data: Vec<u64>, length: u32) {
    set_get_clear(data, length, length);
}
'''),
    'false_clear': ('assertion failed', '''
fn wrong(data: Vec<u64>, length: u32, bit: u32)
    requires length as int <= 64 * data@.len(), bit < length,
{
    let result = set_get_clear(data, length, bit);
    assert(result.2);
}
'''),
    'false_regrow': ('assertion failed', '''
fn wrong(data: Vec<u64>, old_len: u32, small: u32, new_len: u32, query: u32)
    requires old_len as int <= 64 * data@.len(), small <= old_len,
        small <= query < new_len,
{
    let result = shrink_regrow(data, old_len, small, new_len, true, query);
    assert(!result.1);
}
'''),
    'false_goal_after_native': ('assertion failed', '''
fn wrong(data: Vec<u64>) {
    resize_read(data, 0, 1, false, 0);
    assert(false);
}
'''),
}


def materialize(directory):
    directory = Path(directory)
    directory.mkdir(parents=True)
    manifest = {}
    for group, cases in [('bridge', BRIDGE_CASES), ('import', IMPORT_CASES)]:
        for name, (diagnostic, body) in cases.items():
            prefix = 'use vstd::prelude::*;\n'
            if group == 'import':
                prefix += 'extern crate reference88;\nuse reference88::{resize_read, live, set_get_clear, shrink_regrow};\n'
            source = prefix + 'verus! {\n' + body + '\n}\n'
            path = directory / (name + '.rs')
            path.write_text(source)
            manifest[name] = dict(path=str(path.absolute()), sha256=hashlib.sha256(source.encode()).hexdigest(),
                                  diagnostic=diagnostic, group=group,
                                  intended='frontend isolation; zero semantic kill credit' if 'not supported' in diagnostic
                                  else 'backend rejection; confirm intended diagnostic before semantic credit')
    (directory / 'frozen-expectations.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
    return manifest


if __name__ == '__main__':
    print(json.dumps(materialize(sys.argv[1]), indent=2, sort_keys=True))
