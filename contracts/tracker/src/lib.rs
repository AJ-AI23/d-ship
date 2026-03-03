//! Tracker d-app for MultiverseX shipping.
//!
//! Registers tracking events (e.g. picked up, in transit, delivered).
//! Carrier-specific configuration is passed at deployment.
//! Low-sensitivity data (status, timestamp, location) stored in plaintext;
//! optional encrypted payload for receiver-visible sensitive data.
#![no_std]

mod generated_validation {
    include!(concat!(env!("OUT_DIR"), "/validation.rs"));
}

use dship_common::entities::TrackingEventRecord;
use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait Tracker {
    #[view(getConfig)]
    #[storage_mapper("config")]
    fn config(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("carrier_address")]
    fn carrier_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("shipment_contract")]
    fn shipment_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("tracking_events")]
    fn tracking_events(&self, tracking_number: &ManagedBuffer) -> VecMapper<TrackingEventRecord<Self::Api>>;

    #[storage_mapper("agreement_for_shipment")]
    fn agreement_for_shipment(&self, tracking_number: &ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

    #[view(getAgreementForShipment)]
    fn get_agreement_for_shipment(&self, tracking_number: ManagedBuffer) -> ManagedAddress {
        self.agreement_for_shipment(&tracking_number).get()
    }

    #[storage_mapper("encrypted_payload")]
    fn encrypted_payload(&self, tracking_number: &ManagedBuffer) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("forwarder_for_shipment")]
    fn forwarder_for_shipment(&self, tracking_number: &ManagedBuffer) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("pickup_contract")]
    fn pickup_contract(&self) -> SingleValueMapper<ManagedAddress>;

    /// For return shipments: outbound -> reimburse_shipper. Used by Agreement to verify refund eligibility.
    #[storage_mapper("reimbursable_return_for_outbound")]
    fn reimbursable_return_for_outbound(&self, outbound_tracking_number: &ManagedBuffer) -> SingleValueMapper<bool>;

    #[init]
    fn init(&self, config: ManagedBuffer) {
        self.config().set(config);
        self.carrier_address().set(&self.blockchain().get_caller());
    }

    #[upgrade]
    fn upgrade(&self, config: ManagedBuffer) {
        self.config().set(config);
    }

    /// Set the Shipment contract allowed to register events and create shipments.
    /// Only carrier may call.
    #[endpoint(setShipmentContract)]
    fn set_shipment_contract(&self, shipment_contract: ManagedAddress) {
        require!(
            self.blockchain().get_caller() == self.carrier_address().get(),
            "Only carrier may set shipment contract"
        );
        self.shipment_contract().set(&shipment_contract);
    }

    /// Set the Pickup contract allowed to register events.
    /// Only carrier may call.
    #[endpoint(setPickupContract)]
    fn set_pickup_contract(&self, pickup_contract: ManagedAddress) {
        require!(
            self.blockchain().get_caller() == self.carrier_address().get(),
            "Only carrier may set pickup contract"
        );
        self.pickup_contract().set(&pickup_contract);
    }

    /// Register a tracking event for a shipment.
    /// Callable by carrier, the configured Shipment contract, the forwarder for that shipment, or the Pickup contract.
    #[endpoint(registerEvent)]
    fn register_event(
        &self,
        tracking_number: ManagedBuffer,
        event_type: ManagedBuffer,
        timestamp: u64,
        location: OptionalValue<ManagedBuffer>,
    ) {
        let caller = self.blockchain().get_caller();
        let carrier = self.carrier_address().get();
        let shipment_addr = self.shipment_contract().get();
        let forwarder_addr = self.forwarder_for_shipment(&tracking_number).get();
        let pickup_addr = self.pickup_contract().get();
        require!(
            caller == carrier
                || (!shipment_addr.is_zero() && caller == shipment_addr)
                || (!forwarder_addr.is_zero() && caller == forwarder_addr)
                || (!pickup_addr.is_zero() && caller == pickup_addr),
            "Only carrier, shipment contract, forwarder, or pickup may register events"
        );
        require!(
            generated_validation::validate(
                event_type.len(),
                tracking_number.len(),
            ),
            "Empty tracking number or event type"
        );

        let location_buf = match location {
            OptionalValue::Some(l) => l,
            OptionalValue::None => ManagedBuffer::new(),
        };

        let record = TrackingEventRecord {
            event_type,
            timestamp,
            location: location_buf,
        };
        let mut events = self.tracking_events(&tracking_number);
        events.push(&record);
    }

    /// Register initial BOOKED event and optionally set agreement and forwarder for shipment.
    /// Called by Shipment contract when creating a shipment.
    #[endpoint(registerShipment)]
    fn register_shipment(
        &self,
        tracking_number: ManagedBuffer,
        agreement_addr: OptionalValue<ManagedAddress>,
        forwarder_agreement_addr: OptionalValue<ManagedAddress>,
        encrypted_payload: OptionalValue<ManagedBuffer>,
    ) {
        let caller = self.blockchain().get_caller();
        let shipment_addr = self.shipment_contract().get();
        require!(
            !shipment_addr.is_zero() && caller == shipment_addr,
            "Only shipment contract may register shipments"
        );
        require!(
            generated_validation::validate(6, tracking_number.len()),
            "Empty tracking number"
        );

        let book_event = TrackingEventRecord {
            event_type: ManagedBuffer::from(b"BOOKED"),
            timestamp: self.blockchain().get_block_timestamp(),
            location: ManagedBuffer::new(),
        };
        let mut events = self.tracking_events(&tracking_number);
        events.push(&book_event);

        if let OptionalValue::Some(addr) = agreement_addr {
            if !addr.is_zero() {
                self.agreement_for_shipment(&tracking_number).set(&addr);
            }
        }

        if let OptionalValue::Some(addr) = forwarder_agreement_addr {
            if !addr.is_zero() {
                self.forwarder_for_shipment(&tracking_number).set(&addr);
            }
        }

        if let OptionalValue::Some(payload) = encrypted_payload {
            if !payload.is_empty() {
                self.encrypted_payload(&tracking_number).set(&payload);
            }
        }
    }

    /// Register a return shipment. Called by Shipment contract when creating a return.
    /// Records outbound link and reimburse flag for Agreement refund verification.
    #[endpoint(registerReturnShipment)]
    fn register_return_shipment(
        &self,
        return_tracking_number: ManagedBuffer,
        outbound_tracking_number: ManagedBuffer,
        agreement_addr: ManagedAddress,
        reimburse_shipper: bool,
        encrypted_payload: OptionalValue<ManagedBuffer>,
    ) {
        let caller = self.blockchain().get_caller();
        let shipment_addr = self.shipment_contract().get();
        require!(
            !shipment_addr.is_zero() && caller == shipment_addr,
            "Only shipment contract may register return shipments"
        );
        require!(!return_tracking_number.is_empty(), "Empty return tracking number");
        require!(!outbound_tracking_number.is_empty(), "Empty outbound tracking number");
        require!(!agreement_addr.is_zero(), "Agreement address required");

        let book_event = TrackingEventRecord {
            event_type: ManagedBuffer::from(b"BOOKED"),
            timestamp: self.blockchain().get_block_timestamp(),
            location: ManagedBuffer::new(),
        };
        let mut events = self.tracking_events(&return_tracking_number);
        events.push(&book_event);

        self.agreement_for_shipment(&return_tracking_number).set(&agreement_addr);
        // Set true if reimburse; never overwrite true with false (supports multiple returns)
        if reimburse_shipper {
            self.reimbursable_return_for_outbound(&outbound_tracking_number).set(&true);
        }

        if let OptionalValue::Some(payload) = encrypted_payload {
            if !payload.is_empty() {
                self.encrypted_payload(&return_tracking_number).set(&payload);
            }
        }
    }

    /// Returns true if outbound has a return with reimburse_shipper. Used by Agreement.refundForReturnShipment.
    #[view(hasReimbursableReturn)]
    fn has_reimbursable_return(&self, outbound_tracking_number: ManagedBuffer) -> bool {
        !self.reimbursable_return_for_outbound(&outbound_tracking_number).is_empty()
            && self.reimbursable_return_for_outbound(&outbound_tracking_number).get()
    }

    /// Returns plaintext tracking status and events. Accessible by anyone with tracking number.
    #[view(getTrackingStatus)]
    fn get_tracking_status(
        &self,
        tracking_number: ManagedBuffer,
    ) -> MultiValueEncoded<MultiValue3<ManagedBuffer, u64, ManagedBuffer>> {
        let mut result = MultiValueEncoded::new();
        for record in self.tracking_events(&tracking_number).iter() {
            result.push(MultiValue3::from((
                record.event_type,
                record.timestamp,
                record.location,
            )));
        }
        result
    }

    /// Returns encrypted payload for the given tracking number.
    /// Caller decrypts off-chain with their key (K = KDF(tracking_number, receiver_secret)).
    #[view(getEncryptedPayload)]
    fn get_encrypted_payload(&self, tracking_number: ManagedBuffer) -> OptionalValue<ManagedBuffer> {
        if self.encrypted_payload(&tracking_number).is_empty() {
            OptionalValue::None
        } else {
            OptionalValue::Some(self.encrypted_payload(&tracking_number).get())
        }
    }

    /// Returns true if the shipment has a DISPATCHED event. Used by void flow to block refund after pickup.
    #[view(hasDispatched)]
    fn has_dispatched(&self, tracking_number: ManagedBuffer) -> bool {
        self.has_event_type(tracking_number, ManagedBuffer::from(b"DISPATCHED"))
    }

    /// Returns true if the shipment has an event of the given type.
    #[view(hasEventType)]
    fn has_event_type(&self, tracking_number: ManagedBuffer, event_type: ManagedBuffer) -> bool {
        for record in self.tracking_events(&tracking_number).iter() {
            if record.event_type == event_type {
                return true;
            }
        }
        false
    }
}
