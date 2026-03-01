//! Ownership assignment. Caller or defined wallet becomes owner of created entities.

use multiversx_sc::{
    api::ManagedTypeApi,
    types::{ManagedAddress, ManagedBuffer},
};

/// Assigns ownership of an entity to the caller or a specified address.
#[derive(Clone)]
pub struct OwnershipTarget<M: ManagedTypeApi> {
    pub owner: ManagedAddress<M>,
}

impl<M: ManagedTypeApi> OwnershipTarget<M> {
    /// Owner is the caller (transaction sender).
    pub fn caller(caller: &ManagedAddress<M>) -> Self {
        Self {
            owner: caller.clone(),
        }
    }

    /// Owner is a specific address (e.g. from config or payment target).
    pub fn address(owner: ManagedAddress<M>) -> Self {
        Self { owner }
    }
}
