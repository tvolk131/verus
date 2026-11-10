use vstd::prelude::*;

verus! {
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Structural)]
pub enum Permission {
    Unique,
    SharedReadWrite,
    SharedReadOnly,
    Disabled,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug, Structural)]
pub enum AccessKind {
    Read,
    Write,
}

pub open spec fn permission_access_table(permission: Permission, access: AccessKind) -> bool {
    match (permission, access) {
        (Permission::Unique, AccessKind::Read) => true,
        (Permission::Unique, AccessKind::Write) => true,
        (Permission::SharedReadWrite, AccessKind::Read) => true,
        (Permission::SharedReadWrite, AccessKind::Write) => true,
        (Permission::SharedReadOnly, AccessKind::Read) => true,
        (Permission::SharedReadOnly, AccessKind::Write) => false,
        (Permission::Disabled, AccessKind::Read) => false,
        (Permission::Disabled, AccessKind::Write) => false,
    }
}

impl Permission {
    pub fn grants(self, access: AccessKind) -> (r: bool)
        ensures r == permission_access_table(self, access),
    {
        self != Permission::Disabled
            && (access == AccessKind::Read || self != Permission::SharedReadOnly)
    }
}
}
