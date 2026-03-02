//! Service contract: service-specific validation and pricing logic (stub).
//!
//! Exposes validate, quote, and route_key endpoints. Upgrade path: deploy new
//! Service contracts with custom logic and register in Shipment.
#![no_std]

use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait Service {
    #[view(getDefaultPrice)]
    #[storage_mapper("default_price")]
    fn default_price(&self) -> SingleValueMapper<BigUint>;

    #[init]
    fn init(&self, default_price: BigUint) {
        self.default_price().set(&default_price);
    }

    /// Validate shipment payload. Stub: accepts any non-empty payload.
    #[view(validate)]
    fn validate(&self, payload: ManagedBuffer) -> bool {
        !payload.is_empty()
    }

    /// Quote for shipment. Stub: returns default price and hash of payload.
    #[view(quote)]
    fn quote(
        &self,
        payload: ManagedBuffer,
    ) -> MultiValue3<ManagedBuffer, BigUint, ManagedBuffer> {
        let amount = self.default_price().get();
        let quote_hash = self.crypto().keccak256(&payload);
        let quote_hash_buf = ManagedBuffer::from(quote_hash.to_byte_array().as_slice());
        let normalized_metrics = ManagedBuffer::from(b"{}");
        MultiValue3::from((normalized_metrics, amount, quote_hash_buf))
    }

    /// Route key for restrictions. Stub: returns hash of payload.
    #[endpoint(routeKey)]
    fn route_key(&self, payload: ManagedBuffer) -> ManagedBuffer {
        let hash = self.crypto().keccak256(&payload);
        ManagedBuffer::from(hash.to_byte_array().as_slice())
    }
}
