//! Service contract: service-specific validation and pricing logic (stub).
//!
//! Exposes validate, quote, and route_key endpoints. Supports forwarder registry for
//! PUDO/terminal handling. Forwarder lookup/search is done off-chain via indexers;
//! on-chain views support indexer bootstrap and verification only.
#![no_std]

#[allow(dead_code)]
mod generated_validation {
    include!(concat!(env!("OUT_DIR"), "/validation.rs"));
}

use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait Service {
    #[view(getDefaultPrice)]
    #[storage_mapper("default_price")]
    fn default_price(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("carrier_address")]
    fn carrier_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("forwarder_registry")]
    fn forwarder_registry(&self) -> SetMapper<ManagedAddress>;

    #[view(getForwarderDefaultFee)]
    #[storage_mapper("forwarder_default_fee")]
    fn forwarder_default_fee(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("capabilities_config")]
    fn capabilities_config(&self) -> SingleValueMapper<ManagedBuffer>;

    #[init]
    fn init(
        &self,
        default_price: BigUint,
        forwarder_default_fee: OptionalValue<BigUint>,
        capabilities_config: OptionalValue<ManagedBuffer>,
    ) {
        self.default_price().set(&default_price);
        let fee = match forwarder_default_fee {
            OptionalValue::Some(f) => f,
            OptionalValue::None => BigUint::zero(),
        };
        self.forwarder_default_fee().set(&fee);
        self.carrier_address().set(&self.blockchain().get_caller());
        if let OptionalValue::Some(ref cfg) = capabilities_config {
            self.capabilities_config().set(cfg);
        }
    }

    /// Set carrier address. For upgrade path when carrier changes.
    #[endpoint(setCarrierAddress)]
    fn set_carrier_address(&self, carrier: ManagedAddress) {
        require!(
            self.blockchain().get_caller() == self.carrier_address().get(),
            "Only carrier may set carrier address"
        );
        self.carrier_address().set(&carrier);
    }

    /// Register a forwarder agreement. Carrier only.
    /// Validates that the ForwarderAgreement contract exists on-chain before registering.
    #[endpoint(registerForwarder)]
    fn register_forwarder(&self, forwarder_agreement_addr: ManagedAddress) {
        require!(
            self.blockchain().get_caller() == self.carrier_address().get(),
            "Only carrier may register forwarder"
        );
        require!(!forwarder_agreement_addr.is_zero(), "Zero address");

        let owner: ManagedAddress = self
            .forwarder_agreement_proxy(forwarder_agreement_addr.clone())
            .forwarder_owner()
            .execute_on_dest_context();
        require!(!owner.is_zero(), "ForwarderAgreement does not exist on-chain");

        self.forwarder_registry().insert(forwarder_agreement_addr);
    }

    /// Unregister a forwarder. Carrier only.
    #[endpoint(unregisterForwarder)]
    fn unregister_forwarder(&self, forwarder_agreement_addr: ManagedAddress) {
        require!(
            self.blockchain().get_caller() == self.carrier_address().get(),
            "Only carrier may unregister forwarder"
        );
        self.forwarder_registry().remove(&forwarder_agreement_addr);
    }

    /// Validate shipment payload. When forwarder_agreement_addr is provided, verifies it is registered.
    #[view(validate)]
    fn validate(
        &self,
        payload: ManagedBuffer,
        forwarder_agreement_addr: OptionalValue<ManagedAddress>,
    ) -> bool {
        require!(!payload.is_empty(), "Empty payload");

        if let OptionalValue::Some(addr) = forwarder_agreement_addr {
            require!(
                !addr.is_zero(),
                "Invalid forwarder address"
            );
            require!(
                self.forwarder_registry().contains(&addr),
                "Forwarder not registered"
            );
        }

        true
    }

    /// Quote for shipment. Returns (normalized_metrics, total_amount, quote_hash, forwarder_agreement_addr, forwarder_amount).
    /// When forwarder_agreement_addr is provided and registered, adds forwarder_default_fee to total and returns forwarder info.
    #[view(quote)]
    fn quote(
        &self,
        payload: ManagedBuffer,
        forwarder_agreement_addr: OptionalValue<ManagedAddress>,
    ) -> MultiValue5<ManagedBuffer, BigUint, ManagedBuffer, ManagedAddress, BigUint> {
        let mut amount = self.default_price().get();
        let mut fwd_addr = ManagedAddress::zero();
        let mut fwd_amount = BigUint::zero();

        if let OptionalValue::Some(addr) = forwarder_agreement_addr {
            if !addr.is_zero() && self.forwarder_registry().contains(&addr) {
                fwd_amount = self.forwarder_default_fee().get();
                amount += &fwd_amount;
                fwd_addr = addr;
            }
        }

        let quote_hash = self.crypto().keccak256(&payload);
        let quote_hash_buf = ManagedBuffer::from(quote_hash.to_byte_array().as_slice());
        let normalized_metrics = ManagedBuffer::from(b"{}");

        MultiValue5::from((normalized_metrics, amount, quote_hash_buf, fwd_addr, fwd_amount))
    }

    /// Route key for restrictions. Stub: returns hash of payload.
    #[endpoint(routeKey)]
    fn route_key(&self, payload: ManagedBuffer) -> ManagedBuffer {
        let hash = self.crypto().keccak256(&payload);
        ManagedBuffer::from(hash.to_byte_array().as_slice())
    }

    /// Returns total count of registered forwarders. For indexer bootstrap.
    #[view(getRegisteredForwarderCount)]
    fn get_registered_forwarder_count(&self) -> u32 {
        self.forwarder_registry().len() as u32
    }

    /// Returns paginated slice of registered forwarders. For indexer bootstrap.
    /// Lookup/search is done off-chain via indexers.
    #[view(getRegisteredForwarders)]
    fn get_registered_forwarders(
        &self,
        offset: u32,
        limit: u32,
    ) -> MultiValueEncoded<ManagedAddress> {
        let mut result = MultiValueEncoded::new();
        let limit = limit.min(100u32);
        let mut skipped = 0u32;
        let mut count = 0u32;
        for addr in self.forwarder_registry().iter() {
            if skipped < offset {
                skipped += 1;
                continue;
            }
            if count >= limit {
                break;
            }
            result.push(addr);
            count += 1;
        }
        result
    }

    /// Returns all deployment-time configuration. Use for discovery of routes, addons, etc.
    /// Returns (default_price, forwarder_default_fee, carrier_address, capabilities_config).
    /// capabilities_config is an opaque blob (e.g. JSON); empty if not set at init.
    #[view(getCapabilities)]
    fn get_capabilities(&self) -> MultiValue4<BigUint, BigUint, ManagedAddress, ManagedBuffer> {
        let default_price = self.default_price().get();
        let forwarder_default_fee = self.forwarder_default_fee().get();
        let carrier_address = self.carrier_address().get();
        let capabilities_config = if self.capabilities_config().is_empty() {
            ManagedBuffer::new()
        } else {
            self.capabilities_config().get()
        };
        MultiValue4::from((
            default_price,
            forwarder_default_fee,
            carrier_address,
            capabilities_config,
        ))
    }

    #[proxy]
    fn forwarder_agreement_proxy(&self, address: ManagedAddress) -> forwarder_agreement::Proxy<Self::Api>;
}
