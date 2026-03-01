//! Parcel d-app for MultiverseX shipping.
//!
//! This contract template processes parcel management. Carrier-specific configuration
//! is passed at deployment and stored on-chain.
#![no_std]

use multiversx_sc::imports::*;

/// Parcel d-app contract.
/// Configuration is provided at init.
#[multiversx_sc::contract]
pub trait Parcel {
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

    /// Register or update a parcel. Config validation is applied based on stored config.
    #[endpoint]
    fn register_parcel(
        &self,
        _parcel_id: ManagedBuffer,
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
