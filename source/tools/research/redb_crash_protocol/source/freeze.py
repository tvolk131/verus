#!/usr/bin/env python3
"""Authenticate immutable inputs and freeze exact original text before proof trials."""
import hashlib, json, re, shutil
from pathlib import Path
HERE = Path(__file__).resolve().parent
BASE = HERE.parent
CHECKOUT = Path('/tmp/verus-infer-cycle-20261109/applications/redb')
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def span(text, start):
    # Rust brace lexer sufficient for these pinned items; comments/strings do not count.
    depth = 0
    for m in re.finditer(r'//[^\n]*|/\*[\s\S]*?\*/|"(?:\\.|[^"\\])*"|[{}]', text[start:]):
        token = m.group()
        if token == '{': depth += 1
        elif token == '}':
            depth -= 1
            if depth == 0: return start + m.end()
    raise ValueError('unterminated item')
def main():
    cases = json.loads((BASE/'CASES.json').read_text())
    extra = ['src/error.rs', 'src/sync.rs', 'src/lib.rs',
             'src/tree_store/page_store/lru_cache.rs', 'src/tree_store/table_tree.rs',
             'src/tree_store/table_tree_base.rs', 'src/tree_store/page_store/file_backend/mod.rs',
             'src/tree_store/page_store/file_backend/optimized.rs',
             'src/tree_store/page_store/file_backend/fallback.rs']
    hashes = dict(cases['source_sha256'])
    for rel, expected in hashes.items(): assert sha(CHECKOUT/rel) == expected, rel
    for rel in extra: hashes[rel] = sha(CHECKOUT/rel)
    for rel in hashes:
        dst = HERE/'original'/rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(CHECKOUT/rel, dst)
    bundle = Path(cases['accepted_runtime']['bundle'])
    assert sha(bundle/'receipt.json') == cases['accepted_runtime']['receipt_sha256']
    for name, expected in cases['accepted_runtime']['tool_hashes'].items():
        assert sha(bundle/name) == expected, name
    for name, expected in cases['shared_helpers'].items():
        assert sha(HERE.parents[4]/name) == expected, name
    # All occurrences in these production prefixes, including full function bodies and fields.
    selected = {
        'src/transactions.rs': ['WriteTransaction', 'AllocatorStateLatch', 'commit', 'commit_with',
            'commit_inner', 'commit_inner_helper', 'durable_commit', 'process_data_freed_pages_after_commit'],
        'src/tree_store/page_store/page_manager.rs': ['TransactionalMemory', 'InMemoryState',
            'read_header', 'read_shared_header', 'read_file_header', 'begin_writable', 'write_header',
            'clear_recovery_required', 'commit', 'non_durable_commit', 'reload_for_write'],
        'src/tree_store/page_store/cached_file.rs': ['CachedFile', 'PagedCachedFile', 'CheckedBackend',
            'flush', 'flush_write_buffer', 'sync_data', 'write', 'flush_buffered_pages', 'flush_lowest_priority'],
        'src/tree_store/page_store/header.rs': ['DatabaseHeader', 'UnrepairedDatabaseHeader',
            'TransactionHeader', 'from_bytes', 'to_bytes', 'finalize', 'finalize_transaction_slots',
            'select_primary_slot', 'published_slot', 'primary_slot', 'secondary_slot', 'swap_primary_slot',
            'write_secondary_slot'],
        'src/db.rs': ['StorageBackend', 'primary_verifies', 'do_repair', 'verify_primary_checksums',
            'verify_checksums', 'rebuild_allocator_state', 'new', 'get_allocator_state_table'],
    }
    items=[]
    for rel, names in selected.items():
        text=(CHECKOUT/rel).read_text()
        prefix=text.split('#[cfg(test)]')[0]
        pattern=r'(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:fn|struct|enum|trait)\s+(\w+)\b'
        for match in re.finditer(pattern, prefix):
            if match[1] not in names: continue
            start=match.start(); end=span(text,start)
            exact=text[start:end].encode()
            items.append(dict(file=rel,name=match[1],start_line=text[:start].count('\n')+1,
                end_line=text[:end].count('\n')+1,sha256=hashlib.sha256(exact).hexdigest(),bytes=len(exact)))
    manifest=dict(parent='c008326bfed88eab958335412cdc5ec0e9c8130e',redb_commit=cases['redb_commit'],
        source_sha256=hashes,additional_dependencies_registered_before_trials=extra,
        runtime=cases['accepted_runtime'],toolchain=cases['toolchain'],items=items,
        source_coupling_proved=False,trial_status='NOT_RUN',
        note='Full files are the authoritative freeze; item hashes cover complete bodies/fields. No source truncation is proof evidence.')
    (HERE/'FROZEN.json').write_text(json.dumps(manifest,indent=2)+'\n')
if __name__ == '__main__': main()
