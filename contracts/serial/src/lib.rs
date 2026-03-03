//! Serial number generation d-app for MultiverseX shipping.
//!
//! Generates unique serial numbers for shipments. Carrier-specific format
//! and rules are passed via config at deployment.
#![no_std]

#[allow(dead_code)]
mod generated_validation {
    include!(concat!(env!("OUT_DIR"), "/validation.rs"));
}

use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait Serial {
    #[view(getConfig)]
    #[storage_mapper("config")]
    fn config(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("serial_counter")]
    fn serial_counter(&self) -> SingleValueMapper<u64>;

    #[init]
    fn init(&self, config: ManagedBuffer) {
        self.config().set(config);
        self.serial_counter().set(1u64);
    }

    #[upgrade]
    fn upgrade(&self, config: ManagedBuffer) {
        self.config().set(config);
    }

    /// Generate a new serial number. Uses optional prefix + counter.
    /// Format: {prefix}{counter} (e.g. "S12345" for prefix "S").
    #[endpoint]
    fn generate(&self, prefix: OptionalValue<ManagedBuffer>) -> ManagedBuffer {
        let count = self.serial_counter().get();
        self.serial_counter().set(count + 1);

        let counter_buf = sc_format!("{}", count);
        let mut result = ManagedBuffer::new();
        if let OptionalValue::Some(p) = prefix {
            if !p.is_empty() {
                result.append(&p);
            }
        }
        result.append(&counter_buf);
        result
    }
}
