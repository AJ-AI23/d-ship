//! Entity data structures mirroring schemas. Use for on-chain storage and validation.
//!
//! Aligns with: schemas/address, parcel, party, shipment-booking, tracker, etc.

use multiversx_sc::{
    api::ManagedTypeApi,
    types::{BigUint, ManagedAddress, ManagedBuffer, ManagedVec},
};

/// Address location (required: streetName, postalCode, city, countryCode).
/// Mirrors `address.schema.json` location.
#[derive(Clone)]
pub struct Location<M: ManagedTypeApi> {
    pub street_name: ManagedBuffer<M>,
    pub street_number: ManagedBuffer<M>,
    pub postal_code: ManagedBuffer<M>,
    pub city: ManagedBuffer<M>,
    pub country_code: ManagedBuffer<M>,
    pub region: ManagedBuffer<M>,
}

/// Compact address for on-chain use. Full schema may use JSON in config.
#[derive(Clone)]
pub struct Address<M: ManagedTypeApi> {
    pub location: Location<M>,
    pub company_name: ManagedBuffer<M>,
    pub contact_name: ManagedBuffer<M>,
}

/// Party roles per nShift API.
pub struct PartyRoles;

impl PartyRoles {
    pub const SENDER: &'static str = "SENDER";
    pub const RECEIVER: &'static str = "RECEIVER";
    pub const DELIVERY: &'static str = "DELIVERY";
    pub const DISPATCH: &'static str = "DISPATCH";
    pub const RETURN: &'static str = "RETURN";
}

/// Party reference: type (ADDRESS|PUDO) + role + address id.
#[derive(Clone)]
pub struct Party<M: ManagedTypeApi> {
    pub party_type: ManagedBuffer<M>, // "ADDRESS" | "PUDO"
    pub roles: ManagedVec<M, ManagedBuffer<M>>,
    pub address_id: ManagedBuffer<M>,
}

/// Parcel with weight. Mirrors parcel.schema.json.
#[derive(Clone)]
pub struct Parcel<M: ManagedTypeApi> {
    pub reference: ManagedBuffer<M>,
    pub weight_grams: u64,
    pub weight_unit: ManagedBuffer<M>, // "G" | "KG" | "LB" | "OZ"
    pub description: ManagedBuffer<M>,
}

/// Shipment entity stored on-chain.
#[derive(Clone)]
pub struct Shipment<M: ManagedTypeApi> {
    pub tracking_number: ManagedBuffer<M>,
    pub sender_id: ManagedBuffer<M>,
    pub recipient_id: ManagedBuffer<M>,
    pub parcel_ids: ManagedVec<M, ManagedBuffer<M>>,
    pub carrier_definition_id: ManagedBuffer<M>,
    pub service_definition_id: ManagedBuffer<M>,
}

/// Tracking event. Mirrors tracking-event.schema.json.
#[derive(Clone)]
pub struct TrackingEvent<M: ManagedTypeApi> {
    pub tracking_number: ManagedBuffer<M>,
    pub event_type: ManagedBuffer<M>, // BOOKED | DISPATCHED | IN_TRANSIT | etc.
    pub timestamp: u64,
    pub location: ManagedBuffer<M>,
}

/// Stored entity with owner and creation metadata.
#[derive(Clone)]
pub struct OwnedEntity<M: ManagedTypeApi, E: Clone> {
    pub id: ManagedBuffer<M>,
    pub owner: ManagedAddress<M>,
    pub entity: E,
}
