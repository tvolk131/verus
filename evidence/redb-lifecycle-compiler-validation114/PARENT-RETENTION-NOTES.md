# Parent retention execution notes

The first parent-only retention invocation stopped with `KeyError: 'records'`
while reading revision2's manifest. That manifest uses the `files` key, whereas
the worker and review manifests use `records`. No retained archive or document
update had been produced at this point. This was an audit-script schema error,
not a compiler, test, authentication, native, or candidate failure.

The original script is preserved as
`/private/tmp/redb-validation114-parent-retention-attempt1.py`; its failure
traceback is in the tool transcript, not claimed as an on-disk raw log. The
repair explicitly selects the manifest's actual key and retains the same
expected digest and 21-file count. Subsequent retention stdout/stderr are saved.
No additional compiler, test, verifier, or native command was run.
