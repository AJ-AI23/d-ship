//! Entity data structures mirroring schemas. Use for on-chain storage and validation.
//!
//! Aligns with: schemas/address, parcel, party, shipment-booking, tracker, etc.

use multiversx_sc::{
    api::ManagedTypeApi,
    codec::{self, derive::*},
    derive::ManagedVecItem,
    types::{ManagedAddress, ManagedBuffer, ManagedVec},
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
/// When party has DELIVERY role and type PUDO, forwarder_agreement_addr identifies the forwarder.
#[derive(Clone)]
pub struct Party<M: ManagedTypeApi> {
    pub party_type: ManagedBuffer<M>, // "ADDRESS" | "PUDO"
    pub roles: ManagedVec<M, ManagedBuffer<M>>,
    pub address_id: ManagedBuffer<M>,
    /// ForwarderAgreement contract address when this party is the delivery handler.
    pub forwarder_agreement_addr: Option<ManagedAddress<M>>,
}

/// Weight units per parcel.schema.json weightUnit enum.
pub struct ParcelWeightUnit;

impl ParcelWeightUnit {
    pub const G: &'static str = "G";
    pub const KG: &'static str = "KG";
    pub const LB: &'static str = "LB";
    pub const OZ: &'static str = "OZ";
}

/// Dangerous goods item. parcel.schema.json dangerousGoods maxItems: 1.
#[derive(Clone, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct DangerousGoods<M: ManagedTypeApi> {
    pub net_quantity: u64,
    pub net_quantity_unit: ManagedBuffer<M>,
    pub type_code: ManagedBuffer<M>,
    pub quantity: u32,
}

/// Parcel entity mirroring parcel.schema.json.
/// Weight required at parcel level; weightUnit enum: G, KG, LB, OZ.
#[derive(Clone, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct Parcel<M: ManagedTypeApi> {
    pub reference: ManagedBuffer<M>,
    pub description: ManagedBuffer<M>,
    /// Parcel weight (schema: number, minimum 0). Stored as smallest unit (e.g. grams).
    pub weight: u64,
    /// Schema: weightUnit enum ["G","KG","LB","OZ"]
    pub weight_unit: ManagedBuffer<M>,
    /// Schema: itemIds, minItems 1 when present. Empty = not provided.
    pub item_ids: ManagedVec<M, ManagedBuffer<M>>,
    pub serial: ManagedBuffer<M>,
    /// Schema: dangerousGoods maxItems 1. Empty = none.
    pub dangerous_goods: ManagedVec<M, DangerousGoods<M>>,
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

/// Compact tracking event for on-chain storage (tracking_number is the storage key).
#[derive(Clone, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct TrackingEventRecord<M: ManagedTypeApi> {
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
