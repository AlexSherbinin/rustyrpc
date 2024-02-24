use rkyv::{Archive, Deserialize, Serialize};

use crate::protocol;

#[derive(Serialize, Deserialize, Archive)]
#[archive(check_bytes)]
pub enum ServiceKind {
    Public,
    Private,
}

impl From<protocol::ServiceKind> for ServiceKind {
    fn from(value: protocol::ServiceKind) -> Self {
        match value {
            protocol::ServiceKind::Public => Self::Public,
            protocol::ServiceKind::Private => Self::Private,
        }
    }
}

impl From<&ArchivedServiceKind> for protocol::ServiceKind {
    fn from(value: &ArchivedServiceKind) -> Self {
        match value {
            ArchivedServiceKind::Public => Self::Public,
            ArchivedServiceKind::Private => Self::Private,
        }
    }
}
