//! Validation logic aligned with schemas. Use before creating on-chain entities.

use multiversx_sc::{
    api::ManagedTypeApi,
    types::{ManagedBuffer, ManagedVec},
};

/// Config-driven validation limits (from config schemas).
#[derive(Clone)]
pub struct ValidationConfig<M: ManagedTypeApi> {
    pub max_weight_grams: Option<u64>,
    pub min_weight_grams: Option<u64>,
    pub max_width_cm: Option<u64>,
    pub max_height_cm: Option<u64>,
    pub max_length_cm: Option<u64>,
    pub max_parcels: Option<u32>,
    pub allowed_country_codes: Option<ManagedVec<M, ManagedBuffer<M>>>,
}

impl<M: ManagedTypeApi> ValidationConfig<M> {
    /// Parse from JSON config buffer. Returns None if invalid.
    pub fn from_config(_config: &ManagedBuffer<M>) -> Option<Self> {
        // Config parsing: carrier forks can implement JSON parse from multiversx_sc::codec
        // For now, contract passes limits as SC args or uses default
        None
    }
}

/// Validate parcel weight against config.
pub fn validate_parcel_weight(
    weight_grams: u64,
    config: &ValidationConfig<impl ManagedTypeApi>,
) -> bool {
    if let Some(max) = config.max_weight_grams {
        if weight_grams > max {
            return false;
        }
    }
    if let Some(min) = config.min_weight_grams {
        if weight_grams < min {
            return false;
        }
    }
    true
}

/// Validate country code against allowed list.
pub fn validate_country(
    country_code: &ManagedBuffer<impl ManagedTypeApi>,
    config: &ValidationConfig<impl ManagedTypeApi>,
) -> bool {
    match &config.allowed_country_codes {
        None => true,
        Some(allowed) => {
            for cc in allowed.iter() {
                if &cc == country_code {
                    return true;
                }
            }
            false
        }
    }
}

/// Tracker event types (harmonized status).
pub mod event_types {
    pub const BOOKED: &str = "BOOKED";
    pub const DISPATCHED: &str = "DISPATCHED";
    pub const IN_TRANSIT: &str = "IN_TRANSIT";
    pub const OUT_FOR_DELIVERY: &str = "OUT_FOR_DELIVERY";
    pub const DELIVERED: &str = "DELIVERED";
    pub const EXCEPTION: &str = "EXCEPTION";
    pub const VOID: &str = "VOID";
}
