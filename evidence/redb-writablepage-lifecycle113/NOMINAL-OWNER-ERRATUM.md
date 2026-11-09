# Exact nominal owner of the original header write

The historical task shorthand `PageManager::write_header` names the method in
`tree_store/page_store/page_manager.rs`, but the actual recovered source places
that method at1617 inside **impl TransactionalMemory**, beginning at691.
Lane A independently checked the actual source before annotation/probing.

Use `TransactionalMemory::write_header` for future nominal/compiler identity
claims. This correction changes neither the original file/body nor the narrow
scope the user approved. It does not invent a PageManager type, create a native
producer, restore historical path/Git continuity or establish a compiler join.
Earlier labels remain historical shorthand, not evidence of that nominal type.

The projected private verification crate in lane A also has different DefIds
from the original complete redb crate. Its nine byte-identical executable blocks
and original layouts are preservation evidence only. All five applications
failed before any body was checked; no native destructor/glue identity follows.
