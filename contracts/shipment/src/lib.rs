//! Shipment: agreement-aware shipment creation backend.
//!
//! Orchestrates: Service validate/quote -> Agreement authorize/reserve/capture.
#![no_std]

use dship_common::entities;
use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait Shipment {
    #[view(getAllowedFactory)]
    #[storage_mapper("allowed_factory")]
    fn allowed_factory(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("service_registry")]
    fn service_registry(&self, service_id: &ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("shipment")]
    fn shipment(&self, tracking_number: &ManagedBuffer) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("shipment_owner")]
    fn shipment_owner(&self, tracking_number: &ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

    #[init]
    fn init(&self, allowed_factory: ManagedAddress) {
        self.allowed_factory().set(&allowed_factory);
    }

    #[endpoint(registerService)]
    fn register_service(&self, service_id: ManagedBuffer, service_addr: ManagedAddress) {
        self.service_registry(&service_id).set(&service_addr);
    }

    /// Create shipment: validate, quote, authorize, reserve, create, capture.
    #[endpoint(createShipment)]
    fn create_shipment(
        &self,
        agreement_addr: ManagedAddress,
        service_id: ManagedBuffer,
        shipment_payload: ManagedBuffer,
        tracking_number: ManagedBuffer,
    ) {
        let caller = self.blockchain().get_caller();

        let service_addr = self.service_registry(&service_id).get();
        require!(!service_addr.is_zero(), "Service not registered");

        let mut service_proxy = self.service_proxy(service_addr);

        let is_valid: bool = service_proxy.validate(shipment_payload.clone()).execute_on_dest_context();
        require!(is_valid, "Validation failed");

        let (normalized_metrics, amount, quote_hash): (ManagedBuffer, BigUint, ManagedBuffer) =
            service_proxy.quote(shipment_payload.clone()).execute_on_dest_context();

        let mut agreement_proxy = self.agreement_proxy(agreement_addr.clone());

        let authorized: bool = agreement_proxy
            .authorize_shipment(service_id.clone(), normalized_metrics, amount.clone(), quote_hash)
            .execute_on_dest_context();
        require!(authorized, "Authorization failed");

        let reference = tracking_number.clone();
        let reservation_id: u64 = agreement_proxy
            .reserve(amount.clone(), reference)
            .execute_on_dest_context();

        let _entity = entities::Shipment {
            tracking_number: tracking_number.clone(),
            sender_id: ManagedBuffer::new(),
            recipient_id: ManagedBuffer::new(),
            parcel_ids: ManagedVec::new(),
            carrier_definition_id: ManagedBuffer::new(),
            service_definition_id: service_id.clone(),
        };

        self.shipment(&tracking_number).set(&tracking_number);
        self.shipment_owner(&tracking_number).set(&caller);

        let _: () = agreement_proxy.capture(reservation_id).execute_on_dest_context();
    }

    #[proxy]
    fn service_proxy(&self, address: ManagedAddress) -> service::Proxy<Self::Api>;

    #[proxy]
    fn agreement_proxy(&self, address: ManagedAddress) -> agreement::Proxy<Self::Api>;
}
