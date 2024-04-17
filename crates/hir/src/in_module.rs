use crate::hir_def::ModuleId;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct InModule<T> {
    pub module_id: ModuleId,
    pub value: T,
}

impl<T> InModule<T> {
    pub fn new(module_id: ModuleId, value: T) -> InModule<T> {
        InModule { module_id, value }
    }

    pub fn with_value<U>(&self, value: U) -> InModule<U> {
        InModule::new(self.module_id, value)
    }

    pub fn map<F: FnOnce(T) -> U, U>(self, f: F) -> InModule<U> {
        InModule::new(self.module_id, f(self.value))
    }

    pub fn as_ref(&self) -> InModule<&T> {
        self.with_value(&self.value)
    }
}

impl<T: Clone> InModule<&T> {
    pub fn cloned(&self) -> InModule<T> {
        self.with_value(self.value.clone())
    }
}

impl<T> InModule<Option<T>> {
    pub fn transpose(self) -> Option<InModule<T>> {
        Some(InModule::new(self.module_id, self.value?))
    }
}
