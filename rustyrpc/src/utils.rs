use core::ops::{Deref, DerefMut};

use crate::transport;

pub(crate) struct ConnectionCloseOnDrop<Connection: transport::Connection>(pub(crate) Connection);

impl<Connection: transport::Connection> OwnedDroppable for ConnectionCloseOnDrop<Connection> {
    #[allow(clippy::unwrap_used)]
    fn drop_owned(self) {
        tokio::spawn(async move {
            self.0.close().await.unwrap();
        });
    }
}

pub(crate) trait OwnedDroppable {
    fn drop_owned(self);
}

pub(crate) struct DropOwned<T: OwnedDroppable>(Option<T>);

impl<T: OwnedDroppable> From<T> for DropOwned<T> {
    fn from(value: T) -> Self {
        Self(Some(value))
    }
}

impl<T: OwnedDroppable> Deref for DropOwned<T> {
    type Target = T;

    #[allow(clippy::undocumented_unsafe_blocks)]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref().unwrap_unchecked() }
    }
}

impl<T: OwnedDroppable> DerefMut for DropOwned<T> {
    #[allow(clippy::undocumented_unsafe_blocks)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut().unwrap_unchecked() }
    }
}

impl<T: OwnedDroppable> Drop for DropOwned<T> {
    #[allow(clippy::undocumented_unsafe_blocks)]
    fn drop(&mut self) {
        unsafe { self.0.take().unwrap_unchecked() }.drop_owned();
    }
}
