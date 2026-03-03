//! Forwarder Agreement contract: carrier-forwarder relationship where carrier pays forwarder.
//!
//! Represents the on-chain forwarder agreement. The forwarder provides handling services
//! (PUDO, terminal/DC) and receives payment via split capture when shipments use them.
//! Payment flows in via receivePayment when Shipment/Agreement splits the capture.
#![no_std]

#[allow(dead_code)]
mod generated_validation {
    include!(concat!(env!("OUT_DIR"), "/validation.rs"));
}

use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait ForwarderAgreement {
    #[view(getForwarderOwner)]
    #[storage_mapper("forwarder_owner")]
    fn forwarder_owner(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getCarrierAddress)]
    #[storage_mapper("carrier_address")]
    fn carrier_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getCarrierShipmentContract)]
    #[storage_mapper("carrier_shipment_contract")]
    fn carrier_shipment_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getAgreementConfigHash)]
    #[storage_mapper("agreement_config_hash")]
    fn agreement_config_hash(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("enabled_services")]
    fn enabled_services(&self) -> SetMapper<ManagedBuffer>;

    #[init]
    #[allow_multiple_var_args]
    fn init(
        &self,
        forwarder_owner: ManagedAddress,
        carrier_address: ManagedAddress,
        carrier_shipment_contract: ManagedAddress,
        agreement_config_hash: ManagedBuffer,
        enabled_services: MultiValueEncoded<ManagedBuffer>,
    ) {
        self.forwarder_owner().set(&forwarder_owner);
        self.carrier_address().set(&carrier_address);
        self.carrier_shipment_contract().set(&carrier_shipment_contract);
        self.agreement_config_hash().set(&agreement_config_hash);

        for service in enabled_services {
            self.enabled_services().insert(service);
        }
    }

    /// Receive EGLD payment from carrier (split capture). Forwards to forwarder_owner.
    /// Callable by Agreement contract during capture when forwarding a portion to the forwarder.
    #[payable("EGLD")]
    #[endpoint(receivePayment)]
    fn receive_payment(&self) {
        let payment = self.call_value().egld().clone();
        require!(payment > 0, "Zero payment");

        let forwarder = self.forwarder_owner().get();
        self.send().direct_egld(&forwarder, &payment);
    }

    #[view(getEnabledServices)]
    fn get_enabled_services(&self) -> MultiValueEncoded<ManagedBuffer> {
        let mut result = MultiValueEncoded::new();
        for service in self.enabled_services().iter() {
            result.push(service);
        }
        result
    }
}
