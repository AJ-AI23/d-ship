//! Parcel d-app for MultiverseX shipping.
//!
//! Uses parcel.schema.json for entity structure. Validates input and stores
//! the full schema-aligned entity.
#![no_std]

mod generated_validation {
    include!(concat!(env!("OUT_DIR"), "/validation.rs"));
}

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

    #[view(getSerialContract)]
    #[storage_mapper("serial_contract")]
    fn serial_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[init]
    fn init(
        &self,
        config: ManagedBuffer,
        serial_contract: ManagedAddress,
        barcode_format: OptionalValue<ManagedBuffer>,
    ) {
        self.config().set(config);
        self.serial_contract().set(&serial_contract);
        let fmt = match barcode_format {
            OptionalValue::Some(b) if !b.is_empty() => b,
            _ => ManagedBuffer::from(b"CODE128"),
        };
        self.barcode_format().set(&fmt);
    }

    #[upgrade]
    fn upgrade(&self, config: ManagedBuffer, serial_contract: OptionalValue<ManagedAddress>) {
        self.config().set(config);
        if let OptionalValue::Some(addr) = serial_contract {
            if !addr.is_zero() {
                self.serial_contract().set(&addr);
            }
        }
    }

    /// Register a parcel. Input and output follow parcel.schema.json.
    /// Validates weight, weightUnit enum, itemIds min 1 (if provided), dangerousGoods max 1.
    /// Serial: if not provided (OptionalValue::None or empty), generated via Serial contract.
    #[endpoint]
    #[allow_multiple_var_args]
    fn register_parcel(
        &self,
        parcel_id: ManagedBuffer,
        reference: ManagedBuffer,
        description: ManagedBuffer,
        weight: u64,
        weight_unit: ManagedBuffer,
        item_ids: MultiValueEncoded<ManagedBuffer>,
        serial: OptionalValue<ManagedBuffer>,
        dg_net_quantity: u64,
        dg_net_quantity_unit: ManagedBuffer,
        dg_type_code: ManagedBuffer,
        dg_quantity: u32,
    ) {
        let caller = self.blockchain().get_caller();

        let serial_value = match serial {
            OptionalValue::Some(s) if !s.is_empty() => s,
            _ => {
                let serial_addr = self.serial_contract().get();
                require!(!serial_addr.is_zero(), "Serial contract not set");
                let parcel_prefix = ManagedBuffer::from(b"P");
                self.serial_proxy(serial_addr)
                    .generate(OptionalValue::Some(parcel_prefix))
                    .execute_on_dest_context()
            }
        };

        // 1. Build entity from schema-aligned parcel.schema.json structure
        let item_ids_vec: ManagedVec<Self::Api, ManagedBuffer> = item_ids.to_vec();
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
            serial: serial_value,
            dangerous_goods,
        };

        // 2. Validate input against behavior tree (compiled from validation-tree.json)
        let weight_unit_slice = weight_unit.to_boxed_bytes().as_slice();
        require!(
            generated_validation::validate(
                entity.dangerous_goods.len(),
                entity.weight,
                weight_unit_slice,
            ),
            "Parcel validation failed: check weightUnit (G/KG/LB/OZ), itemIds, dangerousGoods"
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

    /// Returns true if the parcel exists on-chain. Used by Shipment to validate parcel_ids.
    #[view(hasParcel)]
    fn has_parcel(&self, parcel_id: ManagedBuffer) -> bool {
        !self.parcel(&parcel_id).is_empty()
    }

    /// Returns label format data for off-chain label generation.
    /// Returns (barcode_format, serial). Barcode format defaults to CODE128.
    #[view(getLabelFormat)]
    fn get_label_format(&self, parcel_id: ManagedBuffer) -> MultiValue2<ManagedBuffer, ManagedBuffer> {
        require!(!self.parcel(&parcel_id).is_empty(), "Parcel not found");
        let entity = self.parcel(&parcel_id).get();
        let barcode_format = if self.barcode_format().is_empty()
            || self.barcode_format().get().is_empty()
        {
            ManagedBuffer::from(b"CODE128")
        } else {
            self.barcode_format().get()
        };
        MultiValue2::from((barcode_format, entity.serial))
    }

    #[storage_mapper("barcode_format")]
    fn barcode_format(&self) -> SingleValueMapper<ManagedBuffer>;

    #[proxy]
    fn serial_proxy(&self, address: ManagedAddress) -> serial::Proxy<Self::Api>;
}
