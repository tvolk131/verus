# Selected-input command receipts

`capture_provenance_receipt.capture(plan, new_receipt_directory)` is an opt-in
standard-library helper. It is not wired into native acceptance, the existing
boundary replay tool, or any build pipeline. The existing boundary replay
receipt already explicitly limits its identity scope; this helper does not
reinterpret that documented limitation as a defect.

## Plan schema

The helper accepts a Python dictionary with exactly these JSON-compatible fields:

| Field | Meaning |
| --- | --- |
| `schema` | Exactly `selected-input-command-plan-v1`. |
| `cwd` | Absolute canonical working directory, already present. |
| `argv` | Nonempty list of strings; first entry is an absolute canonical executable file. No shell expansion. |
| `executable_sha256` | Expected lowercase SHA256 of that executable. |
| `compiler` | `null`, or exactly `{"path": absolute_path, "sha256": expected_digest}`. This is a declared file identity, not an authenticated compiler role. |
| `inventory` | Exactly `root`, `files`, `trees`, and `expected_sha256`. |
| `outputs` | Nonempty, unique absolute required artifact paths; parents already exist. |
| `timeout_seconds` | Finite positive integer/float, not a Boolean. |

Inventory `root` is an absolute canonical directory. `files` and `trees` are
lists of normalized relative names without `..`; at least one selector is
required. Trees recursively select all regular files, including hidden files,
with no implicit extension filter. Symlinked files/directories and nonregular
files are refused. `expected_sha256` is a nonempty mapping of selected relative
file names to lowercase SHA256 values. Its key set and digests must exactly
match the initial selected inventory. Files outside the selection are expressly
uncovered. No environment field or unknown schema fields are accepted.

```python
from pathlib import Path
from capture_provenance_receipt import capture

receipt = capture(plan, Path("/private/tmp/new-command-receipt"))
if not receipt["success"]:
    raise RuntimeError(receipt["errors"])
```

Only trusted commands should be passed. Avoid secrets in command arguments,
selected files, or logs: the helper cannot redact arbitrary command output.
The launch environment is built from exactly `LANG=C`, `LC_ALL=C`, and `TZ=UTC`,
never inherited from the parent. This deliberately omits PATH, HOME, credential
variables, toolchain overrides, flags, and caches; callers must provide an
absolute executable. Programs may modify their own runtime environment.

## Failure and identity semantics

- Malformed dictionaries raise `ValueError` before creating a receipt directory.
  A receipt directory must be new; an existing one is never reused.
- Missing or mismatched selected inputs/executable/declared compiler refuse
  launch. Existing required outputs, including dangling symlinks, refuse launch
  without deleting or overwriting those outputs. All are checked before launch.
- Command exit failure, spawn failure, timeout, missing/nonregular/symlinked
  output, output timestamp older than launch, source inventory/content drift,
  executable/compiler identity drift, or failed rescan yield `success=false`.
  Failures retain observations and logs when available. Outputs from failed
  commands remain diagnostic artifacts, never successful receipts.
- `KeyboardInterrupt` and `SystemExit` record interruption and attempt to kill
  and reap the launched process group before final observations, then propagate
  the original interruption. Success requires a completed zero exit, no timeout
  or interruption, and no recorded errors.
- Source inventory additions/removals are checked within the selected recursive
  trees. Source content drift compares endpoint hashes, not metadata alone.
  Binary identity records include digest, size and modification time.
- Timeout kills the POSIX process group (only the process on other platforms).
  This is not process containment: detached descendants or commands that return
  before their children finish are not controlled or attested.
- Required outputs must be absent before launch and have modification times at
  least the recorded launch wall-clock timestamp. Coarse timestamps or clock
  rollback can cause conservative false negatives. An artifact copied/relinked
  and retimestamped by a command is not distinguished from newly computed bytes.
- Logs are observed at their final pathname, not authenticated against the
  original output file descriptor. Missing, nonregular or symlinked logs refuse
  success; replacement by another regular file is outside this endpoint check.

## What success does not establish

Receipt success means the selected endpoint observations and command outcome
passed these checks, not that a compiler was built or native code was verified.
`vstd-only-command-shape` takes precedence for `--is-vstd`; invoking an existing
compiler on vstd is not an engine build. `engine-build-command-shape` only
recognizes a Cargo build argv naming package `rust_verify`; even its executable
name and successful exit cannot authenticate what the program actually did.
Neither classification supplies a compiler-build count or compilation proof.

Selected source/input hashes, a separately hashed declared compiler file,
executable hashes, and output hashes do **not** establish binary/source
correspondence, transitive toolchain/dependency/cfg/runtime completeness,
uninterrupted custody, an atomic snapshot, absence of transient changes,
reproducibility, trustworthy command behavior, or native semantic proof.
Interpreter/shared-library loading and executable replacement between hashing
and execution remain outside the endpoint observation guarantee. Receipt
authenticity and later artifact validation also require independent custody.

Run the hermetic tests without Cargo, a verifier, solver, or compiler:

```text
python3 -I -B -m unittest discover -s source/tools -p test_capture_provenance_receipt.py -v
```
