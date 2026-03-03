//! Shipment: agreement-aware shipment creation backend.
//!
//! Orchestrates: Service validate/quote -> Agreement authorize/reserve/capture.
//! Validates that agreement, parcel_ids, and forwarder exist on-chain. Lookup is done off-chain via indexers.
#![no_std]

#[allow(dead_code)]
mod generated_validation {
    include!(concat!(env!("OUT_DIR"), "/validation.rs"));
}

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

    /// Returns true if the shipment exists on-chain. Used by Pickup contract for validation.
    #[view(hasShipment)]
    fn has_shipment(&self, tracking_number: ManagedBuffer) -> bool {
        !self.shipment(&tracking_number).is_empty()
    }

    /// Returns true if the shipment has been voided. Used by Pickup and off-chain consumers.
    #[view(isVoided)]
    fn is_voided(&self, tracking_number: ManagedBuffer) -> bool {
        !self.shipment_voided(&tracking_number).is_empty()
    }

    #[storage_mapper("shipment_owner")]
    fn shipment_owner(&self, tracking_number: &ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("shipment_parcel_ids")]
    fn shipment_parcel_ids(&self, tracking_number: &ManagedBuffer) -> SingleValueMapper<ManagedVec<Self::Api, ManagedBuffer>>;

    #[storage_mapper("forwarder_for_shipment")]
    fn forwarder_for_shipment(&self, tracking_number: &ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("shipment_voided")]
    fn shipment_voided(&self, tracking_number: &ManagedBuffer) -> SingleValueMapper<bool>;

    /// For return shipments: return_tn -> outbound_tn (traceability).
    #[storage_mapper("outbound_for_return")]
    fn outbound_for_return(&self, tracking_number: &ManagedBuffer) -> SingleValueMapper<ManagedBuffer>;

    /// For return shipments: return_tn -> reimburse_shipper (exception vs failure to collect).
    #[storage_mapper("return_reimburse_shipper")]
    fn return_reimburse_shipper(&self, tracking_number: &ManagedBuffer) -> SingleValueMapper<bool>;

    /// Reverse lookup: outbound_tn -> return tracking numbers (supports multiple returns).
    #[storage_mapper("return_shipments")]
    fn return_shipments(&self, outbound_tracking_number: &ManagedBuffer) -> SetMapper<ManagedBuffer>;

    #[view(getSerialContract)]
    #[storage_mapper("serial_contract")]
    fn serial_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getTrackerContract)]
    #[storage_mapper("tracker_contract")]
    fn tracker_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getParcelContract)]
    #[storage_mapper("parcel_contract")]
    fn parcel_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getPickupContract)]
    #[storage_mapper("pickup_contract")]
    fn pickup_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[init]
    fn init(
        &self,
        allowed_factory: ManagedAddress,
        serial_contract: ManagedAddress,
        tracker_contract: OptionalValue<ManagedAddress>,
        parcel_contract: OptionalValue<ManagedAddress>,
    ) {
        self.allowed_factory().set(&allowed_factory);
        self.serial_contract().set(&serial_contract);
        if let OptionalValue::Some(addr) = tracker_contract {
            if !addr.is_zero() {
                self.tracker_contract().set(&addr);
            }
        }
        if let OptionalValue::Some(addr) = parcel_contract {
            if !addr.is_zero() {
                self.parcel_contract().set(&addr);
            }
        }
    }

    #[endpoint(registerService)]
    fn register_service(&self, service_id: ManagedBuffer, service_addr: ManagedAddress) {
        self.service_registry(&service_id).set(&service_addr);
    }

    /// Set Parcel contract for parcel existence validation. Deployer/carrier only.
    #[endpoint(setParcelContract)]
    fn set_parcel_contract(&self, parcel_contract: ManagedAddress) {
        self.parcel_contract().set(&parcel_contract);
    }

    /// Set Pickup contract address. Used when configuring Agreements for pickup.
    #[endpoint(setPickupContract)]
    fn set_pickup_contract(&self, pickup_contract: ManagedAddress) {
        self.pickup_contract().set(&pickup_contract);
    }

    /// Returns Service contract address for a given service_id. For indexer lookup.
    #[view(getServiceAddress)]
    fn get_service_address(&self, service_id: ManagedBuffer) -> OptionalValue<ManagedAddress> {
        let addr = self.service_registry(&service_id).get();
        if addr.is_zero() {
            OptionalValue::None
        } else {
            OptionalValue::Some(addr)
        }
    }

    /// Create shipment: validate, quote, authorize, reserve, create, capture.
    /// Generates tracking number via Serial contract. Accepts parcel_ids (must be pre-registered).
    /// Optional forwarder_agreement_addr: when shipment uses a forwarder for delivery (PUDO, etc.).
    /// Optional encrypted_payload: client-side encrypted sensitive data (decrypt with KDF(tracking_number, receiver_secret)).
    #[endpoint(createShipment)]
    #[allow_multiple_var_args]
    fn create_shipment(
        &self,
        agreement_addr: ManagedAddress,
        service_id: ManagedBuffer,
        shipment_payload: ManagedBuffer,
        parcel_ids: MultiValueEncoded<ManagedBuffer>,
        forwarder_agreement_addr: OptionalValue<ManagedAddress>,
        encrypted_payload: OptionalValue<ManagedBuffer>,
    ) {
        let caller = self.blockchain().get_caller();

        let serial_addr = self.serial_contract().get();
        require!(!serial_addr.is_zero(), "Serial contract not set");

        let shipment_prefix = ManagedBuffer::from(b"S");
        let tracking_number: ManagedBuffer = self
            .serial_proxy(serial_addr)
            .generate(OptionalValue::Some(shipment_prefix))
            .execute_on_dest_context();

        let service_addr = self.service_registry(&service_id).get();
        require!(!service_addr.is_zero(), "Service not registered");

        require!(!agreement_addr.is_zero(), "Agreement address required");
        let agreement_shipment: ManagedAddress = self
            .agreement_proxy(agreement_addr.clone())
            .carrier_shipment_contract()
            .execute_on_dest_context();
        require!(
            agreement_shipment == self.blockchain().get_sc_address(),
            "Agreement is not for this Shipment contract"
        );

        let parcel_ids_vec: ManagedVec<Self::Api, ManagedBuffer> = parcel_ids.to_vec();
        let parcel_addr = self.parcel_contract().get();
        if !parcel_addr.is_zero() && !parcel_ids_vec.is_empty() {
            for pid in parcel_ids_vec.iter() {
                let exists: bool = self
                    .parcel_proxy(parcel_addr.clone())
                    .has_parcel(pid.clone())
                    .execute_on_dest_context();
                require!(exists, "Parcel does not exist on-chain");
            }
        }

        let mut service_proxy = self.service_proxy(service_addr);

        let is_valid: bool = service_proxy
            .validate(shipment_payload.clone(), forwarder_agreement_addr.clone())
            .execute_on_dest_context();
        require!(is_valid, "Validation failed");

        let (normalized_metrics, amount, quote_hash, fwd_addr, fwd_amount): (
            ManagedBuffer,
            BigUint,
            ManagedBuffer,
            ManagedAddress,
            BigUint,
        ) = service_proxy
            .quote(shipment_payload.clone(), forwarder_agreement_addr)
            .execute_on_dest_context();

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
            parcel_ids: parcel_ids_vec.clone(),
            carrier_definition_id: ManagedBuffer::new(),
            service_definition_id: service_id.clone(),
        };

        self.shipment(&tracking_number).set(&tracking_number);
        self.shipment_owner(&tracking_number).set(&caller);
        self.shipment_parcel_ids(&tracking_number).set(&parcel_ids_vec);

        if !fwd_addr.is_zero() {
            self.forwarder_for_shipment(&tracking_number).set(&fwd_addr);
        }

        let capture_fwd_addr = if !fwd_addr.is_zero() {
            OptionalValue::Some(fwd_addr.clone())
        } else {
            OptionalValue::None
        };
        let capture_fwd_amount = if fwd_amount > BigUint::zero() {
            OptionalValue::Some(fwd_amount)
        } else {
            OptionalValue::None
        };

        let _: () = agreement_proxy
            .capture(reservation_id, capture_fwd_addr, capture_fwd_amount)
            .execute_on_dest_context();

        let tracker_addr = self.tracker_contract().get();
        if !tracker_addr.is_zero() {
            let agreement_for_tracker = OptionalValue::Some(agreement_addr);
            let fwd_for_tracker = if !fwd_addr.is_zero() {
                OptionalValue::Some(fwd_addr)
            } else {
                OptionalValue::None
            };
            let _: () = self
                .tracker_proxy(tracker_addr)
                .register_shipment(
                    tracking_number,
                    agreement_for_tracker,
                    fwd_for_tracker,
                    encrypted_payload,
                )
                .execute_on_dest_context();
        }
    }

    /// Void a shipment. Callable by shipment owner. Registers VOID event in Tracker.
    /// Refund is available only when voided before dispatch (carrier calls Agreement.refundForVoidedShipment).
    #[endpoint(voidShipment)]
    fn void_shipment(&self, tracking_number: ManagedBuffer) {
        let caller = self.blockchain().get_caller();
        require!(self.has_shipment(tracking_number.clone()), "Shipment does not exist");
        require!(
            caller == self.shipment_owner(&tracking_number).get(),
            "Only shipment owner may void"
        );
        require!(
            self.shipment_voided(&tracking_number).is_empty(),
            "Shipment already voided"
        );

        self.shipment_voided(&tracking_number).set(&true);

        let tracker_addr = self.tracker_contract().get();
        if !tracker_addr.is_zero() {
            let timestamp = self.blockchain().get_block_timestamp();
            let _: () = self
                .tracker_proxy(tracker_addr)
                .register_event(
                    tracking_number,
                    ManagedBuffer::from(b"VOID"),
                    timestamp,
                    OptionalValue::<ManagedBuffer>::None,
                )
                .execute_on_dest_context();
        }
    }

    /// Create a return shipment. Carrier only. References outbound for traceability.
    /// No charge to shipper; reimbursement via Agreement.refundForReturnShipment when reimburse_shipper.
    #[endpoint(createReturnShipment)]
    #[allow_multiple_var_args]
    fn create_return_shipment(
        &self,
        outbound_tracking_number: ManagedBuffer,
        agreement_addr: ManagedAddress,
        service_id: ManagedBuffer,
        shipment_payload: ManagedBuffer,
        parcel_ids: MultiValueEncoded<ManagedBuffer>,
        reimburse_shipper: bool,
        encrypted_payload: OptionalValue<ManagedBuffer>,
    ) {
        let caller = self.blockchain().get_caller();

        require!(self.has_shipment(outbound_tracking_number.clone()), "Outbound shipment does not exist");
        require!(
            self.shipment_voided(&outbound_tracking_number).is_empty(),
            "Cannot return voided shipment"
        );

        let tracker_addr = self.tracker_contract().get();
        require!(!tracker_addr.is_zero(), "Tracker contract not set");

        let has_dispatched: bool = self
            .tracker_proxy(tracker_addr.clone())
            .has_dispatched(outbound_tracking_number.clone())
            .execute_on_dest_context();
        require!(has_dispatched, "Outbound must have been dispatched");

        let agreement_for_outbound: ManagedAddress = self
            .tracker_proxy(tracker_addr.clone())
            .get_agreement_for_shipment(outbound_tracking_number.clone())
            .execute_on_dest_context();
        require!(
            agreement_for_outbound == agreement_addr,
            "Agreement does not match outbound"
        );

        let carrier: ManagedAddress = self
            .agreement_proxy(agreement_addr.clone())
            .carrier_address()
            .execute_on_dest_context();
        require!(caller == carrier, "Only carrier may create return shipment");

        let service_addr = self.service_registry(&service_id).get();
        require!(!service_addr.is_zero(), "Service not registered");

        let parcel_ids_vec: ManagedVec<Self::Api, ManagedBuffer> = parcel_ids.to_vec();
        let parcel_addr = self.parcel_contract().get();
        if !parcel_addr.is_zero() && !parcel_ids_vec.is_empty() {
            for pid in parcel_ids_vec.iter() {
                let exists: bool = self
                    .parcel_proxy(parcel_addr.clone())
                    .has_parcel(pid.clone())
                    .execute_on_dest_context();
                require!(exists, "Parcel does not exist on-chain");
            }
        }

        let is_valid: bool = self
            .service_proxy(service_addr.clone())
            .validate(shipment_payload, OptionalValue::<ManagedAddress>::None)
            .execute_on_dest_context();
        require!(is_valid, "Validation failed");

        let serial_addr = self.serial_contract().get();
        require!(!serial_addr.is_zero(), "Serial contract not set");

        let return_prefix = ManagedBuffer::from(b"R");
        let return_tracking_number: ManagedBuffer = self
            .serial_proxy(serial_addr)
            .generate(OptionalValue::Some(return_prefix))
            .execute_on_dest_context();

        self.shipment(&return_tracking_number).set(&return_tracking_number);
        self.shipment_owner(&return_tracking_number).set(&caller);
        self.shipment_parcel_ids(&return_tracking_number).set(&parcel_ids_vec);
        self.outbound_for_return(&return_tracking_number).set(&outbound_tracking_number);
        self.return_reimburse_shipper(&return_tracking_number).set(&reimburse_shipper);
        self.return_shipments(&outbound_tracking_number).insert(return_tracking_number.clone());

        let _: () = self
            .tracker_proxy(tracker_addr.clone())
            .register_return_shipment(
                return_tracking_number.clone(),
                outbound_tracking_number.clone(),
                agreement_addr,
                reimburse_shipper,
                encrypted_payload,
            )
            .execute_on_dest_context();

        // Register RETURN_INITIATED on outbound for traceability
        let timestamp = self.blockchain().get_block_timestamp();
        let _: () = self
            .tracker_proxy(tracker_addr)
            .register_event(
                outbound_tracking_number,
                ManagedBuffer::from(b"RETURN_INITIATED"),
                timestamp,
                OptionalValue::<ManagedBuffer>::None,
            )
            .execute_on_dest_context();
    }

    /// Returns outbound tracking number for a return. Empty if not a return.
    #[view(getOutboundForReturn)]
    fn get_outbound_for_return(&self, tracking_number: ManagedBuffer) -> OptionalValue<ManagedBuffer> {
        if self.outbound_for_return(&tracking_number).is_empty() {
            OptionalValue::None
        } else {
            OptionalValue::Some(self.outbound_for_return(&tracking_number).get())
        }
    }

    #[proxy]
    fn service_proxy(&self, address: ManagedAddress) -> service::Proxy<Self::Api>;

    #[proxy]
    fn agreement_proxy(&self, address: ManagedAddress) -> agreement::Proxy<Self::Api>;

    #[proxy]
    fn serial_proxy(&self, address: ManagedAddress) -> serial::Proxy<Self::Api>;

    #[proxy]
    fn tracker_proxy(&self, address: ManagedAddress) -> tracker::Proxy<Self::Api>;

    #[proxy]
    fn parcel_proxy(&self, address: ManagedAddress) -> parcel::Proxy<Self::Api>;
}
