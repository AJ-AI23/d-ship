//! Billing and reserve-capture state machine.

use multiversx_sc::{
    api::ManagedTypeApi,
    codec::{self, derive::*},
    derive::type_abi,
    types::{BigUint, ManagedBuffer},
};

/// Reservation state in the reserve-capture flow.
#[type_abi]
#[derive(Clone, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq)]
pub enum ReservationState {
    Reserved,
    Captured,
    Released,
}

/// A reservation record for funds held during shipment creation.
#[type_abi]
#[derive(Clone, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct Reservation<M: ManagedTypeApi> {
    pub id: u64,
    pub amount: BigUint<M>,
    pub reference: ManagedBuffer<M>,
    pub state: ReservationState,
}

impl<M: ManagedTypeApi> Reservation<M> {
    pub fn new(id: u64, amount: BigUint<M>, reference: ManagedBuffer<M>) -> Self {
        Self {
            id,
            amount,
            reference,
            state: ReservationState::Reserved,
        }
    }

    pub fn is_reserved(&self) -> bool {
        self.state == ReservationState::Reserved
    }

    pub fn is_captured(&self) -> bool {
        self.state == ReservationState::Captured
    }

    pub fn is_released(&self) -> bool {
        self.state == ReservationState::Released
    }
}
