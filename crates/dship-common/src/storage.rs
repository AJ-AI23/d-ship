//! Split entity data into multiple storage units. Use for large or modular entities.

use multiversx_sc::{
    api::ManagedTypeApi,
    types::{ManagedAddress, ManagedBuffer},
};

/// Storage unit key pattern for sharded entities.
/// e.g. "shipment:{"id"}", "shipment:{"id"}:parcels:{"idx"}"
#[derive(Clone)]
pub struct StorageKey<M: ManagedTypeApi> {
    pub prefix: ManagedBuffer<M>,
    pub id: ManagedBuffer<M>,
    pub suffix: ManagedBuffer<M>,
}

impl<M: ManagedTypeApi> StorageKey<M> {
    pub fn entity(entity_type: &str, id: &ManagedBuffer<M>) -> Self {
        Self {
            prefix: ManagedBuffer::from(entity_type),
            id: id.clone(),
            suffix: ManagedBuffer::new(),
        }
    }

    pub fn with_suffix(mut self, suffix: &str) -> Self {
        self.suffix = ManagedBuffer::from(suffix);
        self
    }

    /// Build key for sub-unit (e.g. parcels in a shipment).
    pub fn sub(&self, sub_type: &str, sub_id: &ManagedBuffer<M>) -> Self {
        Self {
            prefix: self.prefix.clone(),
            id: sub_id.clone(),
            suffix: ManagedBuffer::from(sub_type),
        }
    }
}
