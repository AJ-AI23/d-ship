//! Parcel d-app for MultiverseX shipping.
//!
//! Uses dship-common for schema-aligned entities, validation, ownership, payment.
#![no_std]

use dship_common::{entities, ownership, validation};
use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait Parcel {
    #[view(getConfig)]
    #[storage_mapper("config")]
    fn config(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("parcel")]
    fn parcel(&self, id: &ManagedBuffer) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("owner")]
    fn owner(&self, id: &ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

    #[init]
    fn init(&self, config: ManagedBuffer) {
        self.config().set(config);
    }

    #[upgrade]
    fn upgrade(&self, config: ManagedBuffer) {
        self.config().set(config);
    }

    /// Register a parcel. Validates weight, assigns ownership.
    #[endpoint]
    fn register_parcel(
        &self,
        parcel_id: ManagedBuffer,
        reference: ManagedBuffer,
        weight_grams: u64,
        weight_unit: ManagedBuffer,
    ) {
        let caller = self.blockchain().get_caller();

        // 1. Process input: build entity from schema-aligned Parcel
        let entity = entities::Parcel {
            reference: reference.clone(),
            weight_grams,
            weight_unit,
            description: ManagedBuffer::new(),
        };

        // 2. Validation against config
        let config = self.config().get();
        let validation_cfg = validation::ValidationConfig::from_config(&config);
        if let Some(ref v) = validation_cfg {
            require!(
                validation::validate_parcel_weight(weight_grams, v),
                "Weight validation failed"
            );
        }

        // 3. Ownership: assign to caller
        let ownership_target = ownership::OwnershipTarget::caller(&caller);
        self.owner(&parcel_id).set(&ownership_target.owner);

        // 4. Storage
        self.parcel(&parcel_id).set(&parcel_id);
    }

    #[view(getConfigHash)]
    fn get_config_hash(&self) -> ManagedBuffer {
        self.config().get()
    }
}
