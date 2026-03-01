//! Charge the caller or a defined payment wallet. EGLD or ESDT.

use multiversx_sc::{
    api::ManagedTypeApi,
    types::{BigUint, ManagedAddress, ManagedBuffer},
};

/// Payment configuration: who pays and how much.
#[derive(Clone)]
pub struct PaymentConfig<M: ManagedTypeApi> {
    /// Recipient of the charge (can be caller or platform wallet).
    pub payment_target: ManagedAddress<M>,
    /// Amount in wei (1 EGLD = 10^18).
    pub amount: BigUint<M>,
    /// Optional ESDT token; empty = EGLD.
    pub token: ManagedBuffer<M>,
}

impl<M: ManagedTypeApi> PaymentConfig<M> {
    /// Charge goes to a specific wallet (e.g. platform).
    pub fn to_wallet(target: ManagedAddress<M>, amount: BigUint<M>) -> Self {
        Self {
            payment_target: target,
            amount,
            token: ManagedBuffer::new(),
        }
    }

    /// No charge.
    pub fn free() -> Self {
        Self {
            payment_target: ManagedAddress::zero(),
            amount: BigUint::zero(),
            token: ManagedBuffer::new(),
        }
    }
}
