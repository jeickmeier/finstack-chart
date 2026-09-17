//! Shared versioned installer map for trusted extension implementations.
use super::{ExtensionDescriptor, OperationRef, extensions};
use crate::{ChartResult, DiagnosticCode};
use std::{collections::BTreeMap, fmt::Debug};

#[derive(Clone)]
pub(crate) struct VersionedEntry<T> {
    pub descriptor: ExtensionDescriptor,
    pub implementation: T,
}

impl<T> Debug for VersionedEntry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.descriptor.fmt(f)
    }
}

#[derive(Clone)]
pub(crate) struct VersionedMap<T> {
    entries: BTreeMap<(String, u64), VersionedEntry<T>>,
}

impl<T> Debug for VersionedMap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VersionedMap")
            .field("entries", &self.entries)
            .finish()
    }
}

impl<T> Default for VersionedMap<T> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

impl<T> VersionedMap<T> {
    pub fn insert(
        &mut self,
        descriptor: ExtensionDescriptor,
        implementation: T,
        already: &str,
        budget: &str,
    ) -> ChartResult<()> {
        extensions::validate_descriptor(&descriptor)?;
        let key = (
            descriptor.operation.id.clone(),
            descriptor.operation.version.get(),
        );
        if self.entries.contains_key(&key) {
            return Err(super::error(DiagnosticCode::SchemaConflict, already));
        }
        crate::limits::require_within(self.entries.len() < 64, budget)?;
        self.entries.insert(
            key,
            VersionedEntry {
                descriptor,
                implementation,
            },
        );
        Ok(())
    }

    pub fn get(&self, operation: &OperationRef, missing: &str) -> ChartResult<&VersionedEntry<T>> {
        self.entries
            .get(&(operation.id.clone(), operation.version.get()))
            .ok_or_else(|| super::error(DiagnosticCode::UnsupportedCapability, missing))
    }

    pub fn get_fmt(
        &self,
        operation: &OperationRef,
        missing: impl FnOnce() -> String,
    ) -> ChartResult<&VersionedEntry<T>> {
        self.entries
            .get(&(operation.id.clone(), operation.version.get()))
            .ok_or_else(|| super::error(DiagnosticCode::UnsupportedCapability, missing()))
    }
}

pub(crate) fn reject_native_only(
    portable: bool,
    is_portable: bool,
    message: &str,
) -> ChartResult<()> {
    if portable && !is_portable {
        Err(super::error(DiagnosticCode::UnsupportedCapability, message))
    } else {
        Ok(())
    }
}
