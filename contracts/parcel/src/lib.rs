//! Parcel d-app for MultiverseX shipping.
//!
//! Uses parcel.schema.json for entity structure. Validates input and stores
//! the full schema-aligned entity.
#![no_std]

use dship_common::{entities, ownership, validation};
use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait Parcel {
    #[view(getConfig)]
    #[storage_mapper("config")]
    fn config(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("parcel")]
    fn parcel(&self, id: &ManagedBuffer) -> SingleValueMapper<entities::Parcel<Self::Api>>;

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

    /// Register a parcel. Input and output follow parcel.schema.json.
    /// Validates weight, weightUnit enum, itemIds min 1 (if provided), dangerousGoods max 1.
    #[endpoint]
    fn register_parcel(
        &self,
        parcel_id: ManagedBuffer,
        reference: ManagedBuffer,
        description: ManagedBuffer,
        weight: u64,
        weight_unit: ManagedBuffer,
        #[var_args] item_ids: MultiValueEncoded<ManagedBuffer>,
        serial: ManagedBuffer,
        dg_net_quantity: u64,
        dg_net_quantity_unit: ManagedBuffer,
        dg_type_code: ManagedBuffer,
        dg_quantity: u32,
    ) {
        let caller = self.blockchain().get_caller();

        // 1. Build entity from schema-aligned parcel.schema.json structure
        let item_ids_vec: ManagedVec<_, ManagedBuffer> = item_ids.to_vec();
        let dangerous_goods = if dg_net_quantity > 0 {
            let mut dg = ManagedVec::new();
            dg.push(entities::DangerousGoods {
                net_quantity: dg_net_quantity,
                net_quantity_unit: dg_net_quantity_unit,
                type_code: dg_type_code,
                quantity: dg_quantity,
            });
            dg
        } else {
            ManagedVec::new()
        };

        let entity = entities::Parcel {
            reference: reference.clone(),
            description,
            weight,
            weight_unit: weight_unit.clone(),
            item_ids: item_ids_vec,
            serial,
            dangerous_goods,
        };

        // 2. Validate input against parcel.schema.json rules
        require!(
            validation::validate_parcel(&entity),
            "Parcel validation failed: check weightUnit (G/KG/LB/OZ), itemIds, dangerousGoods"
        );
        require!(
            validation::validate_weight_unit(&weight_unit),
            "Invalid weightUnit; use G, KG, LB, or OZ"
        );

        // 3. Config-driven validation (weight limits)
        let config = self.config().get();
        let validation_cfg = validation::ValidationConfig::from_config(&config);
        if let Some(ref v) = validation_cfg {
            require!(
                validation::validate_parcel_weight(weight, v),
                "Weight validation failed"
            );
        }

        // 4. Ownership: assign to caller
        let ownership_target = ownership::OwnershipTarget::caller(&caller);
        self.owner(&parcel_id).set(&ownership_target.owner);

        // 5. Store the full schema entity
        self.parcel(&parcel_id).set(&entity);
    }

    #[view(getConfigHash)]
    fn get_config_hash(&self) -> ManagedBuffer {
        self.config().get()
    }
}
