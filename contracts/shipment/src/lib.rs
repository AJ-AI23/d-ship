//! Shipment d-app for MultiverseX shipping.
//!
//! This contract template processes shipment creation. Carrier-specific configuration
//! (e.g. validators, processors) is passed at deployment and stored on-chain.
#![no_std]

use multiversx_sc::imports::*;

/// Shipment d-app contract.
/// Configuration (validators, processors, network settings) is provided at init.
#[multiversx_sc::contract]
pub trait Shipment {
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

    /// Create a new shipment. Config validation is applied based on stored config.
    #[endpoint]
    fn create_shipment(
        &self,
        _tracking_number: ManagedBuffer,
        _sender_address: ManagedAddress,
        _recipient_address: ManagedAddress,
    ) {
        // Template: actual validation against config happens in carrier forks
        // Config is available via self.config().get()
    }

    #[view(getConfigHash)]
    fn get_config_hash(&self) -> ManagedBuffer {
        let config = self.config().get();
        // Placeholder: config hash for verification
        config
    }
}
