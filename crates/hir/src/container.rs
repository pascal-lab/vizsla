use utils::impl_from;

use crate::hir_def::{block::BlockId, ModuleId};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ContainerId {
    ModuleId(ModuleId),
    BlockId(BlockId),
}

impl_from!(ModuleId, BlockId for ContainerId);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct InContainer<T> {
    pub container_id: ContainerId,
    pub value: T,
}

impl<T> InContainer<T> {
    pub fn new(container_id: ContainerId, value: T) -> InContainer<T> {
        InContainer { container_id, value }
    }

    pub fn with_value<U>(&self, value: U) -> InContainer<U> {
        InContainer::new(self.container_id, value)
    }

    pub fn map<F: FnOnce(T) -> U, U>(self, f: F) -> InContainer<U> {
        InContainer::new(self.container_id, f(self.value))
    }

    pub fn as_ref(&self) -> InContainer<&T> {
        self.with_value(&self.value)
    }
}

impl<T: Clone> InContainer<&T> {
    pub fn cloned(&self) -> InContainer<T> {
        self.with_value(self.value.clone())
    }
}

impl<T> InContainer<Option<T>> {
    pub fn transpose(self) -> Option<InContainer<T>> {
        Some(InContainer::new(self.container_id, self.value?))
    }
}
