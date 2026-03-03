//! Agreement contract: per-customer agreement and billing authority.
//!
//! Represents the on-chain customer agreement. Only the bound carrier shipment contract
//! may reserve, capture, or release funds.
#![no_std]

#[allow(dead_code)]
mod generated_validation {
    include!(concat!(env!("OUT_DIR"), "/validation.rs"));
}

use dship_common::billing::{Reservation, ReservationState};
use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait Agreement {
    #[view(getCustomerOwner)]
    #[storage_mapper("customer_owner")]
    fn customer_owner(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getCarrierAddress)]
    #[storage_mapper("carrier_address")]
    fn carrier_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getCarrierShipmentContract)]
    #[storage_mapper("carrier_shipment_contract")]
    fn carrier_shipment_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getAgreementConfigHash)]
    #[storage_mapper("agreement_config_hash")]
    fn agreement_config_hash(&self) -> SingleValueMapper<ManagedBuffer>;

    #[view(getDepositBalance)]
    #[storage_mapper("deposit_balance")]
    fn deposit_balance(&self) -> SingleValueMapper<BigUint>;

    #[view(getCreditLimit)]
    #[storage_mapper("credit_limit")]
    fn credit_limit(&self) -> SingleValueMapper<BigUint>;

    #[view(getReservedAmount)]
    #[storage_mapper("reserved_amount")]
    fn reserved_amount(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("reservation_id_counter")]
    fn reservation_id_counter(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("reservation")]
    fn reservation(&self, id: &u64) -> SingleValueMapper<Reservation<Self::Api>>;

    #[storage_mapper("enabled_services")]
    fn enabled_services(&self) -> SetMapper<ManagedBuffer>;

    #[storage_mapper("pickup_contract")]
    fn pickup_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("pickup_fee")]
    fn pickup_fee(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("pickup_slots_hash")]
    fn pickup_slots_hash(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("tracker_contract")]
    fn tracker_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("captured_for_shipment")]
    fn captured_for_shipment(&self, tracking_number: &ManagedBuffer) -> SingleValueMapper<BigUint>;

    #[view(getPickupContract)]
    fn get_pickup_contract(&self) -> ManagedAddress {
        self.pickup_contract().get()
    }

    #[view(getPickupFee)]
    fn get_pickup_fee(&self) -> BigUint {
        self.pickup_fee().get()
    }

    #[view(getPickupSlotsHash)]
    fn get_pickup_slots_hash(&self) -> ManagedBuffer {
        self.pickup_slots_hash().get()
    }

    #[init]
    #[allow_multiple_var_args]
    fn init(
        &self,
        customer_owner: ManagedAddress,
        carrier_address: ManagedAddress,
        carrier_shipment_contract: ManagedAddress,
        agreement_config_hash: ManagedBuffer,
        credit_limit: BigUint,
        pickup_contract: ManagedAddress,
        pickup_fee: BigUint,
        pickup_slots_hash: ManagedBuffer,
        enabled_services: MultiValueEncoded<ManagedBuffer>,
    ) {
        self.customer_owner().set(&customer_owner);
        self.carrier_address().set(&carrier_address);
        self.carrier_shipment_contract().set(&carrier_shipment_contract);
        self.agreement_config_hash().set(&agreement_config_hash);
        self.deposit_balance().set(&BigUint::zero());
        self.reserved_amount().set(&BigUint::zero());
        self.reservation_id_counter().set(1u64);

        self.credit_limit().set(&credit_limit);

        if !pickup_contract.is_zero() {
            self.pickup_contract().set(&pickup_contract);
        }
        if pickup_fee > BigUint::zero() {
            self.pickup_fee().set(&pickup_fee);
        }
        if !pickup_slots_hash.is_empty() {
            self.pickup_slots_hash().set(&pickup_slots_hash);
        }

        for service in enabled_services {
            self.enabled_services().insert(service);
        }
    }

    /// Deposit EGLD into the customer account. Any caller can fund (funds go to customer).
    #[payable("EGLD")]
    #[endpoint]
    fn deposit(&self) {
        let payment = self.call_value().egld().clone();
        require!(payment > 0, "Zero deposit");
        let current = self.deposit_balance().get();
        self.deposit_balance().set(&(current + payment));
    }

    /// Authorize a pickup. View-like; validates amount within available balance.
    /// Called by pickup contract before reserve.
    #[view(authorizePickup)]
    fn authorize_pickup(&self, amount: BigUint) -> bool {
        require!(amount > 0, "Zero amount");

        let deposit = self.deposit_balance().get();
        let reserved = self.reserved_amount().get();
        let credit = self.credit_limit().get();
        let available = deposit - &reserved + &credit;
        require!(amount <= available, "Insufficient balance or credit");

        true
    }

    /// Set pickup contract. Only carrier_shipment_contract may call.
    #[endpoint(setPickupContract)]
    fn set_pickup_contract(&self, pickup_contract: ManagedAddress) {
        require!(
            self.blockchain().get_caller() == self.carrier_shipment_contract().get(),
            "Only shipment contract may set pickup contract"
        );
        self.pickup_contract().set(&pickup_contract);
    }

    /// Set pickup fee. Only carrier_shipment_contract may call.
    #[endpoint(setPickupFee)]
    fn set_pickup_fee(&self, amount: BigUint) {
        require!(
            self.blockchain().get_caller() == self.carrier_shipment_contract().get(),
            "Only shipment contract may set pickup fee"
        );
        self.pickup_fee().set(&amount);
    }

    /// Set pickup slots hash for on-chain slot validation. Only carrier_shipment_contract may call.
    #[endpoint(setPickupSlotsHash)]
    fn set_pickup_slots_hash(&self, hash: ManagedBuffer) {
        require!(
            self.blockchain().get_caller() == self.carrier_shipment_contract().get(),
            "Only shipment contract may set pickup slots hash"
        );
        self.pickup_slots_hash().set(&hash);
    }

    /// Set Tracker contract for refund validation (hasDispatched, hasEventType). Only carrier_shipment_contract may call.
    #[endpoint(setTrackerContract)]
    fn set_tracker_contract(&self, tracker_contract: ManagedAddress) {
        require!(
            self.blockchain().get_caller() == self.carrier_shipment_contract().get(),
            "Only shipment contract may set tracker contract"
        );
        self.tracker_contract().set(&tracker_contract);
    }

    /// Authorize a shipment. View-like; validates service, amount within limits.
    /// Called by shipment contract during create_shipment.
    #[view(authorizeShipment)]
    fn authorize_shipment(
        &self,
        service_id: ManagedBuffer,
        _normalized_metrics: ManagedBuffer,
        amount: BigUint,
        _quote_hash: ManagedBuffer,
    ) -> bool {
        require!(self.enabled_services().contains(&service_id), "Service not enabled");
        require!(amount > 0, "Zero amount");

        let deposit = self.deposit_balance().get();
        let reserved = self.reserved_amount().get();
        let credit = self.credit_limit().get();
        let available = deposit - &reserved + &credit;
        require!(amount <= available, "Insufficient balance or credit");

        true
    }

    /// Reserve funds. Only callable by shipment or pickup contract.
    #[endpoint]
    fn reserve(&self, amount: BigUint, reference: ManagedBuffer) -> u64 {
        let caller = self.blockchain().get_caller();
        let shipment = self.carrier_shipment_contract().get();
        let pickup_addr = self.pickup_contract().get();
        require!(
            caller == shipment
                || (!pickup_addr.is_zero() && caller == pickup_addr),
            "Only shipment or pickup contract may reserve"
        );
        require!(amount > 0, "Zero reserve");

        let deposit = self.deposit_balance().get();
        let reserved = self.reserved_amount().get();
        let credit = self.credit_limit().get();
        let available = deposit.clone() - reserved.clone() + credit;
        require!(amount <= available, "Insufficient balance or credit");

        let id = self.reservation_id_counter().get();
        self.reservation_id_counter().set(id + 1);

        let reservation = Reservation::new(id, amount.clone(), reference);
        self.reservation(&id).set(&reservation);
        self.reserved_amount().set(&(reserved + amount));

        id
    }

    /// Capture a reservation. Only callable by shipment or pickup contract.
    /// Transfers the reserved amount to the carrier. When forwarder params provided, splits:
    /// (amount - forwarder_amount) to carrier, forwarder_amount to forwarder via receivePayment.
    #[endpoint]
    #[allow_multiple_var_args]
    fn capture(
        &self,
        reservation_id: u64,
        forwarder_agreement_addr: OptionalValue<ManagedAddress>,
        forwarder_amount: OptionalValue<BigUint>,
    ) {
        let caller = self.blockchain().get_caller();
        let shipment = self.carrier_shipment_contract().get();
        let pickup_addr = self.pickup_contract().get();
        require!(
            caller == shipment
                || (!pickup_addr.is_zero() && caller == pickup_addr),
            "Only shipment or pickup contract may capture"
        );

        let mut reservation = self.reservation(&reservation_id).get();
        require!(reservation.is_reserved(), "Reservation not in Reserved state");

        let amount = reservation.amount.clone();
        reservation.state = ReservationState::Captured;
        self.reservation(&reservation_id).set(&reservation);

        let reserved = self.reserved_amount().get();
        self.reserved_amount().set(&(reserved - &amount));

        let deposit = self.deposit_balance().get();
        self.deposit_balance().set(&(deposit - &amount));

        let carrier = self.carrier_address().get();

        let (carrier_amount, fwd_addr, fwd_amount) = match (&forwarder_agreement_addr, &forwarder_amount) {
            (OptionalValue::Some(addr), OptionalValue::Some(amt)) if !addr.is_zero() && *amt > BigUint::zero() && *amt < amount => {
                let carrier_amt = &amount - amt;
                (carrier_amt, addr.clone(), amt.clone())
            }
            _ => (amount, ManagedAddress::zero(), BigUint::zero()),
        };

        self.send().direct_egld(&carrier, &carrier_amount);

        if !fwd_addr.is_zero() && fwd_amount > BigUint::zero() {
            let owner: ManagedAddress = self
                .forwarder_agreement_proxy(fwd_addr.clone())
                .forwarder_owner()
                .execute_on_dest_context();
            require!(!owner.is_zero(), "ForwarderAgreement does not exist on-chain");

            let _: () = self
                .forwarder_agreement_proxy(fwd_addr)
                .receive_payment()
                .with_egld_transfer(fwd_amount)
                .execute_on_dest_context();
        }

        // Store captured amount for shipment refunds (only for shipment creates, not pickups)
        let ref_bytes = reservation.reference.to_boxed_bytes();
        if !ref_bytes.as_slice().starts_with(b"PICKUP|") {
            self.captured_for_shipment(&reservation.reference).set(&reservation.amount);
        }
    }

    /// Refund a voided shipment to the customer. Callable by carrier with EGLD equal to captured amount.
    /// Requires: shipment has VOID event, no DISPATCHED event, amount matches captured_for_shipment.
    #[payable("EGLD")]
    #[endpoint(refundForVoidedShipment)]
    fn refund_for_voided_shipment(&self, tracking_number: ManagedBuffer) {
        require!(
            self.blockchain().get_caller() == self.carrier_address().get(),
            "Only carrier may refund"
        );
        let payment = self.call_value().egld().clone();
        require!(payment > 0, "Zero refund");

        let expected = self.captured_for_shipment(&tracking_number).get();
        require!(expected > BigUint::zero(), "No captured amount for this shipment");
        require!(payment == expected, "Refund amount must match captured amount");

        let tracker_addr = self.tracker_contract().get();
        require!(!tracker_addr.is_zero(), "Tracker contract not set");

        let has_void: bool = self
            .tracker_proxy(tracker_addr.clone())
            .has_event_type(tracking_number.clone(), ManagedBuffer::from(b"VOID"))
            .execute_on_dest_context();
        require!(has_void, "Shipment must be voided");

        let has_dispatched: bool = self
            .tracker_proxy(tracker_addr)
            .has_dispatched(tracking_number.clone())
            .execute_on_dest_context();
        require!(!has_dispatched, "Cannot refund after dispatch");

        self.captured_for_shipment(&tracking_number).clear();
        let deposit = self.deposit_balance().get();
        self.deposit_balance().set(&(deposit + payment));
    }

    /// Refund outbound shipment cost when return is due to exception. Callable by carrier with EGLD.
    /// Requires: Tracker has reimbursable return for outbound, amount matches captured_for_shipment.
    #[payable("EGLD")]
    #[endpoint(refundForReturnShipment)]
    fn refund_for_return_shipment(&self, outbound_tracking_number: ManagedBuffer) {
        require!(
            self.blockchain().get_caller() == self.carrier_address().get(),
            "Only carrier may refund"
        );
        let payment = self.call_value().egld().clone();
        require!(payment > 0, "Zero refund");

        let expected = self.captured_for_shipment(&outbound_tracking_number).get();
        require!(expected > BigUint::zero(), "No captured amount for this shipment");
        require!(payment == expected, "Refund amount must match captured amount");

        let tracker_addr = self.tracker_contract().get();
        require!(!tracker_addr.is_zero(), "Tracker contract not set");

        let has_reimbursable: bool = self
            .tracker_proxy(tracker_addr)
            .has_reimbursable_return(outbound_tracking_number.clone())
            .execute_on_dest_context();
        require!(has_reimbursable, "No reimbursable return for this outbound");

        self.captured_for_shipment(&outbound_tracking_number).clear();
        let deposit = self.deposit_balance().get();
        self.deposit_balance().set(&(deposit + payment));
    }

    /// Release a reservation. Only callable by shipment or pickup contract.
    #[endpoint]
    fn release(&self, reservation_id: u64) {
        let caller = self.blockchain().get_caller();
        let shipment = self.carrier_shipment_contract().get();
        let pickup_addr = self.pickup_contract().get();
        require!(
            caller == shipment
                || (!pickup_addr.is_zero() && caller == pickup_addr),
            "Only shipment or pickup contract may release"
        );

        let mut reservation = self.reservation(&reservation_id).get();
        require!(reservation.is_reserved(), "Reservation not in Reserved state");
        reservation.state = ReservationState::Released;
        self.reservation(&reservation_id).set(&reservation);

        let reserved = self.reserved_amount().get();
        self.reserved_amount().set(&(reserved - reservation.amount));
    }

    #[proxy]
    fn forwarder_agreement_proxy(&self, address: ManagedAddress) -> forwarder_agreement::Proxy<Self::Api>;

    #[proxy]
    fn tracker_proxy(&self, address: ManagedAddress) -> tracker::Proxy<Self::Api>;

    #[view(getReservation)]
    fn get_reservation(&self, reservation_id: u64) -> OptionalValue<Reservation<Self::Api>> {
        if self.reservation(&reservation_id).is_empty() {
            OptionalValue::None
        } else {
            OptionalValue::Some(self.reservation(&reservation_id).get())
        }
    }
}
