#!/usr/bin/env python3
"""Independent compiler probes for 55bd67ad..3b05c191; logs stay outside git."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

PREFIX = "use vstd::prelude::*;\nverus! {\n"
CASES = {
    "eager_then_some": (False, """
#[verifier::infer] pub fn f(c:bool,x:u8)->Option<u8>{c.then_some(x+1)}
fn bad(){let _ = f(false,255);}
"""),
    "eager_map_default": (False, """
#[verifier::infer] pub fn f(o:Option<u8>,x:u8)->u8{o.map_or(x+1,|v|v)}
fn bad(){let _ = f(Some(0),255);}
"""),
    "inactive_callback": (True, """
#[verifier::infer] pub fn f(o:Option<u8>)->u8{o.map_or(7,|v|v+1)}
fn good()->(r:u8) ensures r==7,{f(None)}
"""),
    "active_callback": (False, """
#[verifier::infer] pub fn f(o:Option<u8>)->u8{o.map_or(7,|v|v+1)}
fn bad(){let _=f(Some(255));}
"""),
    "captured_callback": (True, """
#[verifier::infer] pub fn f(o:Option<u8>,x:u8)->u8{o.map_or(9,|v|v+x)}
fn good(x:u8)->(r:u8) ensures r==9,{f(None,x)}
fn sum(v:u8,x:u8)->(r:u8) requires v as int+x<=255, ensures r==v as int+x,{f(Some(v),x)}
"""),
    "array_tail": (False, """
#[verifier::infer] pub fn f(o:Option<u64>)->[u8;8]{o.map_or(0u64.to_le_bytes(),|v|v.to_le_bytes())}
fn bad(){let a=f(Some(72057594037927936));assert(a[7]==0);}
"""),
    "array_empty": (True, """
#[verifier::infer] pub fn f(o:Option<u8>)->[u8;0]{o.map_or([],|v|[])}
fn good(){let a=f(Some(255));assert(a@.len()==0);}
"""),
    "conversion_signed": (True, """
#[verifier::infer] pub fn f(x:i128)->i8{i8::try_from(x).unwrap()}
fn good(x:i128)->(r:i8) requires -128<=x<=127, ensures r==x,{f(x)}
"""),
    "conversion_negative": (False, """
#[verifier::infer] pub fn f(x:i128)->i8{i8::try_from(x).unwrap()}
fn bad(){let _=f(-129);}
"""),
    "conversion_large_unsigned": (False, """
#[verifier::infer] pub fn f(x:u128)->i64{i64::try_from(x).unwrap()}
fn bad(){let _=f(9223372036854775808);}
"""),
    "conversion_option": (True, """
#[verifier::infer] pub fn f(o:Option<i64>)->u8{o.map_or(4,|v|u8::try_from(v).unwrap())}
fn good()->(r:u8) ensures r==4,{f(None)}
fn narrow(x:i64)->(r:u8) requires 0<=x<=255, ensures r==x,{f(Some(x))}
"""),
    "conversion_option_panic": (False, """
#[verifier::infer] pub fn f(o:Option<i64>)->u8{o.map_or(4,|v|u8::try_from(v).unwrap())}
fn bad(){let _=f(Some(-1));}
"""),
    "nested_record_updates": (True, """
pub struct R{pub x:u64}
impl R{#[verifier::infer] pub fn f(self)->[u8;8]{let mut n=self.x;{n|=256;}n.to_le_bytes()}}
fn good(){let a=R{x:0}.f();assert(a[1]==1);}
"""),
    "nested_multi_field_updates": (True, """
pub struct R{pub x:u64,pub tag:bool}
impl R{#[verifier::infer] pub fn f(self)->[u8;8]{let mut n=self.x;{n|=256;}n.to_le_bytes()}}
fn good(){let a=R{x:0,tag:false}.f();assert(a[1]==1);}
"""),
    "record_operand_effect": (False, """
pub struct R{pub x:u64}
impl R{#[verifier::infer] pub fn f(self)->[u8;8]{let mut n=self.x;let v={n=256;n};(n|v).to_le_bytes()}}
"""),
    "record_early_panic": (True, """
pub struct R{pub x:u8}
impl R{#[verifier::infer] pub fn f(&self,c:bool)->u8{if c{return self.x;}panic!("stop")}}
fn good(x:u8)->(r:u8) ensures r==x,{R{x}.f(true)}
"""),
    "record_early_panic_call": (False, """
pub struct R{pub x:u8}
impl R{#[verifier::infer] pub fn f(&self,c:bool)->u8{if c{return self.x;}panic!("stop")}}
fn bad(){let _=R{x:0}.f(false);}
"""),
    "shift_zero": (False, """
pub struct R{pub order:u8}
impl R{#[verifier::infer] pub fn f(&self,size:u32)->u64{(1u64<<self.order)*u64::from(size)}}
fn bad(){let _=R{order:64}.f(0);}
"""),
    "shift_large": (True, """
pub struct R{pub order:u8}
impl R{#[verifier::infer] pub fn f(&self,size:u32)->u64{(1u64<<self.order)*u64::from(size)}}
fn good()->(r:u64) ensures r==9223372036854775808,{R{order:63}.f(1)}
"""),
    "mutable_alias_true": (True, """
#[verifier::infer] pub fn mask(x:u8)->u8{x&15}
fn good(x:u8){let mut input=0u8;let y=mask(input);input=x;assert(y==0);}
"""),
    "mutable_alias_false": (False, """
#[verifier::infer] pub fn mask(x:u8)->u8{x&15}
fn bad(x:u8){let mut input=x;let y=mask(input);input=0;assert(y==0);}
"""),
    "branch_alias_true": (True, """
#[verifier::infer] pub fn mask(x:u8)->u8{x&15}
fn good(x:u8,c:bool){let mut input=0u8;let y=mask(input);if c{input=x;assert(y==0);}}
"""),
    "return_alias_true": (True, """
#[verifier::infer] pub fn mask(x:u8)->u8{x&15}
fn good(x:u8)->(r:u8) ensures r==0,{let mut input=0u8;let y=mask(input);input=x;y}
"""),
    "return_alias_false": (False, """
#[verifier::infer] pub fn mask(x:u8)->u8{x&15}
fn bad(x:u8)->(r:u8) ensures r==0,{let mut input=x;let y=mask(input);input=0;y}
"""),
    "return_alias_equation_true": (True, """
#[verifier::infer] pub fn mask(x:u8)->u8{x&15}
fn good(x:u8)->(r:u8) ensures r==(0u8&15),{let mut input=0u8;let y=mask(input);input=x;y}
"""),
    "return_alias_equation_false": (False, """
#[verifier::infer] pub fn mask(x:u8)->u8{x&15}
fn bad(x:u8)->(r:u8) ensures r==(0u8&15),{let mut input=x;let y=mask(input);input=0;y}
"""),
    "callback_branch": (True, """
#[verifier::infer] pub fn f(o:Option<u8>,c:bool)->u8{o.map_or(0,|v|if c{v+1}else{v})}
fn good(v:u8)->(r:u8) ensures r==v,{f(Some(v),false)}
"""),
    "callback_contract": (False, """
#[verifier::infer] pub fn f(o:Option<u8>)->u8{o.map_or(0,|v:u8|->(r:u8) requires v<100, ensures r==v,{v})}
"""),
}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--verus", type=Path, required=True)
    parser.add_argument("--solver", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--case", action="append", choices=CASES)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    environment = dict(os.environ, VERUS_Z3_PATH=str(args.solver.resolve()))
    environment.pop("VERUS_INFER_CACHE", None)
    receipt = {"verus_sha256": hashlib.sha256(args.verus.read_bytes()).hexdigest(),
               "solver_sha256": hashlib.sha256(args.solver.read_bytes()).hexdigest(), "runs": []}
    for name in args.case or CASES:
        expected, body = CASES[name]
        source = args.output / (name + ".rs")
        source.write_text(PREFIX + body + "}\n")
        for assisted in [False, True]:
            label = name + ("_goals" if assisted else "_plain")
            artifact = args.output / (label + ".vir")
            if not expected:
                artifact.write_bytes(b"seeded stale review interface")
            argv = [str(args.verus.resolve()), str(source.resolve()), "--crate-type=lib",
                    "--no-cheating", "--output-json", "--num-threads", "1",
                    "--export", str(artifact.resolve())]
            if assisted:
                argv.append("--infer-goals")
            try:
                run = subprocess.run(argv, env=environment, text=True, capture_output=True, timeout=120)
                code, stdout, stderr = run.returncode, run.stdout, run.stderr
            except subprocess.TimeoutExpired as error:
                code, stdout, stderr = None, str(error.stdout), str(error.stderr)
            (args.output / (label + ".stdout")).write_text(stdout)
            (args.output / (label + ".stderr")).write_text(stderr)
            entry = dict(case=name, assisted=assisted, expected=expected, exit_code=code,
                         exported=artifact.exists(), stale_seeded=not expected, argv=argv,
                         source_sha256=hashlib.sha256(source.read_bytes()).hexdigest())
            try:
                data = json.loads(stdout)
                entry["verification"] = data.get("verification-results")
                entry["inference"] = data.get("verus-infer")
            except (ValueError, TypeError):
                pass
            receipt["runs"].append(entry)
            (args.output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
            print(label, code, "expected", expected, flush=True)


if __name__ == "__main__":
    main()
