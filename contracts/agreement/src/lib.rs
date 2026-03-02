//! Agreement contract: per-customer agreement and billing authority.
//!
//! Represents the on-chain customer agreement. Only the bound carrier shipment contract
//! may reserve, capture, or release funds.
#![no_std]

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

    #[init]
    #[allow_multiple_var_args]
    fn init(
        &self,
        customer_owner: ManagedAddress,
        carrier_address: ManagedAddress,
        carrier_shipment_contract: ManagedAddress,
        agreement_config_hash: ManagedBuffer,
        credit_limit: BigUint,
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

    /// Reserve funds. Only callable by shipment contract.
    #[endpoint]
    fn reserve(&self, amount: BigUint, reference: ManagedBuffer) -> u64 {
        let caller = self.blockchain().get_caller();
        require!(
            caller == self.carrier_shipment_contract().get(),
            "Only shipment contract may reserve"
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

    /// Capture a reservation. Only callable by shipment contract.
    /// Transfers the reserved amount to the carrier.
    #[endpoint]
    fn capture(&self, reservation_id: u64) {
        let caller = self.blockchain().get_caller();
        require!(
            caller == self.carrier_shipment_contract().get(),
            "Only shipment contract may capture"
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
        self.send().direct_egld(&carrier, &amount);
    }

    /// Release a reservation. Only callable by shipment contract.
    #[endpoint]
    fn release(&self, reservation_id: u64) {
        let caller = self.blockchain().get_caller();
        require!(
            caller == self.carrier_shipment_contract().get(),
            "Only shipment contract may release"
        );

        let mut reservation = self.reservation(&reservation_id).get();
        require!(reservation.is_reserved(), "Reservation not in Reserved state");
        reservation.state = ReservationState::Released;
        self.reservation(&reservation_id).set(&reservation);

        let reserved = self.reserved_amount().get();
        self.reserved_amount().set(&(reserved - reservation.amount));
    }

    #[view(getReservation)]
    fn get_reservation(&self, reservation_id: u64) -> OptionalValue<Reservation<Self::Api>> {
        if self.reservation(&reservation_id).is_empty() {
            OptionalValue::None
        } else {
            OptionalValue::Some(self.reservation(&reservation_id).get())
        }
    }
}
