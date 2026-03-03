//! Agreement configuration and approval message format.
//!
//! Aligns with `schemas/agreement-config.schema.json`.
//! Approval message format: `APPROVE_DEPLOY|{carrierAddr}|{factoryAddr}|{codeHash}|{configHash}|{expiry}|{nonce}`
//! Uses pipe delimiter. Addresses and hashes as raw bytes; expiry/nonce as 8-byte big-endian.

use multiversx_sc::{
    api::ManagedTypeApi,
    types::{BigUint, ManagedAddress, ManagedBuffer, ManagedVec},
};

/// Agreement configuration (schema-aligned). Stored off-chain; only hash on-chain.
#[derive(Clone)]
pub struct AgreementConfig<M: ManagedTypeApi> {
    pub enabled_services: ManagedVec<M, ManagedBuffer<M>>,
    pub route_restrictions: Option<RouteRestrictions<M>>,
    pub quotas: Option<Quotas<M>>,
    pub pricing_model_ref: Option<ManagedBuffer<M>>,
    pub credit_limit: Option<BigUint<M>>,
    pub spending_cap: Option<BigUint<M>>,
}

/// Origin/destination country filters.
#[derive(Clone)]
pub struct RouteRestrictions<M: ManagedTypeApi> {
    pub allowed_origin_countries: Option<ManagedVec<M, ManagedBuffer<M>>>,
    pub allowed_destination_countries: Option<ManagedVec<M, ManagedBuffer<M>>>,
}

/// Quota limits.
#[derive(Clone)]
pub struct Quotas<M: ManagedTypeApi> {
    pub max_shipments_per_month: Option<u32>,
    pub max_amount_per_month: Option<BigUint<M>>,
}

/// Builds the approval message that the customer signs for deployment.
/// Format: APPROVE_DEPLOY|{carrier_addr}|{factory_addr}|{code_hash}|{config_hash}|{expiry_8be}{nonce_8be}
/// Expiry and nonce as 8-byte big-endian (no delimiter between them).
/// The contract should hash this (keccak256 or sha256) and verify the signature.
pub fn build_approval_message<M: ManagedTypeApi>(
    carrier_addr: &ManagedAddress<M>,
    factory_addr: &ManagedAddress<M>,
    customer_account_code_hash: &ManagedBuffer<M>,
    agreement_config_hash: &ManagedBuffer<M>,
    expiry: u64,
    nonce: u64,
) -> ManagedBuffer<M> {
    let mut msg = ManagedBuffer::new();
    msg.append_bytes(b"APPROVE_DEPLOY|");
    msg.append(carrier_addr.as_managed_buffer());
    msg.append_bytes(b"|");
    msg.append(factory_addr.as_managed_buffer());
    msg.append_bytes(b"|");
    msg.append(customer_account_code_hash);
    msg.append_bytes(b"|");
    msg.append(agreement_config_hash);
    msg.append_bytes(b"|");
    msg.append_bytes(&expiry.to_be_bytes());
    msg.append_bytes(&nonce.to_be_bytes());
    msg
}

/// Builds the approval message that the forwarder signs for ForwarderAgreement deployment.
/// Format: APPROVE_FORWARDER_DEPLOY|{carrier_addr}|{factory_addr}|{code_hash}|{config_hash}|{expiry_8be}{nonce_8be}
/// Same structure as customer approval; prefix distinguishes forwarder flow.
pub fn build_forwarder_approval_message<M: ManagedTypeApi>(
    carrier_addr: &ManagedAddress<M>,
    factory_addr: &ManagedAddress<M>,
    forwarder_account_code_hash: &ManagedBuffer<M>,
    agreement_config_hash: &ManagedBuffer<M>,
    expiry: u64,
    nonce: u64,
) -> ManagedBuffer<M> {
    let mut msg = ManagedBuffer::new();
    msg.append_bytes(b"APPROVE_FORWARDER_DEPLOY|");
    msg.append(carrier_addr.as_managed_buffer());
    msg.append_bytes(b"|");
    msg.append(factory_addr.as_managed_buffer());
    msg.append_bytes(b"|");
    msg.append(forwarder_account_code_hash);
    msg.append_bytes(b"|");
    msg.append(agreement_config_hash);
    msg.append_bytes(b"|");
    msg.append_bytes(&expiry.to_be_bytes());
    msg.append_bytes(&nonce.to_be_bytes());
    msg
}
