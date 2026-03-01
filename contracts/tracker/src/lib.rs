//! Tracker d-app for MultiverseX shipping.
//!
//! Registers tracking events (e.g. picked up, in transit, delivered).
//! Carrier-specific configuration is passed at deployment.
#![no_std]

use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait Tracker {
    #[view(getConfig)]
    #[storage_mapper("config")]
    fn config(&self) -> SingleValueMapper<ManagedBuffer>;

    #[init]
    fn init(&self, config: ManagedBuffer) {
        self.config().set(config);
    }

    #[upgrade]
    fn upgrade(&self, config: ManagedBuffer) {
        self.config().set(config);
    }

    /// Register a tracking event for a shipment.
    #[endpoint]
    fn register_event(
        &self,
        _tracking_number: ManagedBuffer,
        _event_type: ManagedBuffer,
        _timestamp: u64,
        _location: OptionalValue<ManagedBuffer>,
    ) {
        // Template: carrier forks implement validation against config
    }
}
