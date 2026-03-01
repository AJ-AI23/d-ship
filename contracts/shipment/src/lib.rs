//! Shipment d-app for MultiverseX shipping.
//!
//! Uses dship-common for: entity types (schema-aligned), validation, ownership,
//! payment, and access control.
#![no_std]

use dship_common::{access, entities, ownership, storage, validation};
use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait Shipment {
    #[view(getConfig)]
    #[storage_mapper("config")]
    fn config(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("payment_target")]
    fn payment_target(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("shipment")]
    fn shipment(&self, id: &ManagedBuffer) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("owner")]
    fn owner(&self, id: &ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

    #[init]
    fn init(&self, config: ManagedBuffer) {
        self.config().set(config);
    }

    #[endpoint(setPaymentTarget)]
    fn set_payment_target(&self, target: ManagedAddress) {
        self.payment_target().set(target);
    }

    #[upgrade]
    fn upgrade(&self, config: ManagedBuffer) {
        self.config().set(config);
    }

    /// Create a shipment. Validates input, assigns ownership, optionally charges.
    #[payable("EGLD")]
    #[endpoint]
    fn create_shipment(
        &self,
        tracking_number: ManagedBuffer,
        sender_id: ManagedBuffer,
        recipient_id: ManagedBuffer,
        #[var_args] parcel_ids: MultiValueEncoded<ManagedBuffer>,
    ) {
        let caller = self.blockchain().get_caller();

        // 1. Process input: build entity from schema-aligned types
        let parcel_ids_vec: ManagedVec<_, ManagedBuffer> = parcel_ids.to_vec();
        let entity = entities::Shipment {
            tracking_number: tracking_number.clone(),
            sender_id,
            recipient_id,
            parcel_ids: parcel_ids_vec,
            carrier_definition_id: ManagedBuffer::new(),
            service_definition_id: ManagedBuffer::new(),
        };

        // 2. Validation (config-driven when ValidationConfig::from_config is implemented)
        let config = self.config().get();
        let validation_cfg = validation::ValidationConfig::from_config(&config);
        if let Some(ref v) = validation_cfg {
            if let Some(max) = v.max_parcels {
                require!(entity.parcel_ids.len() <= max as usize, "Too many parcels");
            }
        }

        // 3. Ownership: assign to caller
        let ownership_target = ownership::OwnershipTarget::caller(&caller);
        self.owner(&tracking_number).set(&ownership_target.owner);

        // 4. Access control: caller must be allowed (ownership handles create)
        let resource = ManagedBuffer::from("shipment");
        require!(
            access::can_perform(
                &caller,
                &ManagedBuffer::from("create"),
                &resource,
                &ownership_target.owner,
            ),
            "Access denied"
        );

        // 5. Storage: entity by id (extend with storage::StorageKey for sub-units)
        self.shipment(&tracking_number).set(&tracking_number);

        // 6. Charge: if payment_target configured, forward EGLD
        if let Some(target) = self.payment_target().get() {
            let amount = self.call_value().egld_value();
            if amount > 0u32 {
                self.send().direct_egld(&target, &amount);
            }
        }
    }

    #[view(getConfigHash)]
    fn get_config_hash(&self) -> ManagedBuffer {
        self.config().get()
    }
}
