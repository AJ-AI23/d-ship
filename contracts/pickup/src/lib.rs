//! Pickup contract: schedules pickup after shipment creation.
//!
//! Accepts multiple tracking numbers, validates they exist and belong to the given agreement,
//! charges the shipper via Agreement reserve/capture, and registers DISPATCHED events in Tracker.
#![no_std]

mod generated_validation {
    include!(concat!(env!("OUT_DIR"), "/validation.rs"));
}

use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait Pickup {
    #[view(getShipmentContract)]
    #[storage_mapper("shipment_contract")]
    fn shipment_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getTrackerContract)]
    #[storage_mapper("tracker_contract")]
    fn tracker_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getCarrierAddress)]
    #[storage_mapper("carrier_address")]
    fn carrier_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getPickupDefaultFee)]
    #[storage_mapper("pickup_default_fee")]
    fn pickup_default_fee(&self) -> SingleValueMapper<BigUint>;

    #[init]
    fn init(
        &self,
        shipment_contract: ManagedAddress,
        tracker_contract: ManagedAddress,
        pickup_default_fee: OptionalValue<BigUint>,
    ) {
        self.shipment_contract().set(&shipment_contract);
        self.tracker_contract().set(&tracker_contract);
        self.carrier_address().set(&self.blockchain().get_caller());
        let fee = match pickup_default_fee {
            OptionalValue::Some(f) => f,
            OptionalValue::None => BigUint::zero(),
        };
        self.pickup_default_fee().set(&fee);
    }

    /// Request pickup for multiple shipments. All must belong to the same agreement.
    /// Charges the shipper via Agreement reserve/capture; registers DISPATCHED events.
    #[endpoint(requestPickup)]
    #[allow_multiple_var_args]
    fn request_pickup(
        &self,
        tracking_numbers: MultiValueEncoded<ManagedBuffer>,
        agreement_addr: ManagedAddress,
        slot_date: u64,
        slot_time_from: ManagedBuffer,
        slot_time_to: ManagedBuffer,
        location: OptionalValue<ManagedBuffer>,
    ) {
        let tracking_vec: ManagedVec<Self::Api, ManagedBuffer> = tracking_numbers.to_vec();
        require!(!tracking_vec.is_empty(), "At least one tracking number required");
        require!(
            generated_validation::validate(tracking_vec.len() as u64),
            "Batch size exceeds limit"
        );
        require!(!agreement_addr.is_zero(), "Agreement address required");

        let shipment_addr = self.shipment_contract().get();
        require!(!shipment_addr.is_zero(), "Shipment contract not set");

        let tracker_addr = self.tracker_contract().get();
        require!(!tracker_addr.is_zero(), "Tracker contract not set");

        let mut agreement_proxy = self.agreement_proxy(agreement_addr.clone());
        let pickup_contract_addr: ManagedAddress = agreement_proxy.get_pickup_contract().execute_on_dest_context();
        require!(
            pickup_contract_addr == self.blockchain().get_sc_address(),
            "Pickup contract not authorized for this agreement"
        );

        let mut tracker_proxy = self.tracker_proxy(tracker_addr.clone());

        for tn in tracking_vec.iter() {
            let exists: bool = self
                .shipment_proxy(shipment_addr.clone())
                .has_shipment(tn.clone())
                .execute_on_dest_context();
            require!(exists, "Shipment does not exist on-chain");

            let voided: bool = self
                .shipment_proxy(shipment_addr.clone())
                .is_voided(tn.clone())
                .execute_on_dest_context();
            require!(!voided, "Shipment is voided");

            let agreement_for_shipment: ManagedAddress = tracker_proxy
                .get_agreement_for_shipment(tn.clone())
                .execute_on_dest_context();
            require!(
                agreement_for_shipment == agreement_addr,
                "Shipment does not belong to this agreement"
            );
        }

        let mut pickup_fee = agreement_proxy.get_pickup_fee().execute_on_dest_context();
        if pickup_fee == BigUint::zero() {
            pickup_fee = self.pickup_default_fee().get();
        }
        require!(pickup_fee > BigUint::zero(), "Pickup fee not configured");

        let authorized: bool = agreement_proxy
            .authorize_pickup(pickup_fee.clone())
            .execute_on_dest_context();
        require!(authorized, "Authorization failed");

        let reference = self.build_pickup_reference(&tracking_vec, slot_date, &slot_time_from, &slot_time_to);
        let reservation_id: u64 = agreement_proxy
            .reserve(pickup_fee.clone(), reference)
            .execute_on_dest_context();

        let mut agreement_proxy = self.agreement_proxy(agreement_addr.clone());
        let timestamp = self.blockchain().get_block_timestamp();
        let location_buf = match &location {
            OptionalValue::Some(l) => l.clone(),
            OptionalValue::None => ManagedBuffer::new(),
        };
        let event_type = ManagedBuffer::from(b"DISPATCHED");

        for tn in tracking_vec.iter() {
            let _: () = tracker_proxy
                .register_event(tn.clone(), event_type.clone(), timestamp, OptionalValue::Some(location_buf.clone()))
                .execute_on_dest_context();
        }

        let _: () = agreement_proxy
            .capture(
                reservation_id,
                OptionalValue::<ManagedAddress>::None,
                OptionalValue::<BigUint>::None,
            )
            .execute_on_dest_context();
    }

    fn build_pickup_reference(
        &self,
        tracking_numbers: &ManagedVec<Self::Api, ManagedBuffer>,
        slot_date: u64,
        slot_time_from: &ManagedBuffer,
        slot_time_to: &ManagedBuffer,
    ) -> ManagedBuffer {
        let mut ref_buf = ManagedBuffer::new();
        ref_buf.append_bytes(b"PICKUP|");
        ref_buf.append_bytes(&slot_date.to_be_bytes());
        ref_buf.append_bytes(b"|");
        ref_buf.append(slot_time_from);
        ref_buf.append_bytes(b"-");
        ref_buf.append(slot_time_to);
        ref_buf.append_bytes(b"|");
        for tn in tracking_numbers.iter() {
            ref_buf.append(&tn);
            ref_buf.append_bytes(b";");
        }
        ref_buf
    }

    #[proxy]
    fn agreement_proxy(&self, address: ManagedAddress) -> agreement::Proxy<Self::Api>;

    #[proxy]
    fn shipment_proxy(&self, address: ManagedAddress) -> shipment::Proxy<Self::Api>;

    #[proxy]
    fn tracker_proxy(&self, address: ManagedAddress) -> tracker::Proxy<Self::Api>;
}
