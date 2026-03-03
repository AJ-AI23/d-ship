//! Tracking event code definitions.
//!
//! Harmonized event types (simplified status) and definition types (info, exception, milestone).
//! Aligns with openPEPPOL TransportationStatusCode, X12 EDI 157, ISO/IEC 19987 (EPCIS).
//!
//! Use `harmonized_event_type` for display/aggregation; use `definition_type` for semantic category;
//! use `codified_meaning` for machine interoperability with external systems.

/// Harmonized event types – simplified high-level status for UI and aggregation.
pub mod harmonized {
    pub const BOOKED: &str = "BOOKED";
    pub const DISPATCHED: &str = "DISPATCHED";
    pub const IN_TRANSIT: &str = "IN_TRANSIT";
    pub const OUT_FOR_DELIVERY: &str = "OUT_FOR_DELIVERY";
    pub const DELIVERED: &str = "DELIVERED";
    pub const EXCEPTION: &str = "EXCEPTION";
    pub const VOID: &str = "VOID";
    pub const RETURN_INITIATED: &str = "RETURN_INITIATED";

    /// All valid harmonized event types.
    pub const ALL: &[&str] = &[
        BOOKED,
        DISPATCHED,
        IN_TRANSIT,
        OUT_FOR_DELIVERY,
        DELIVERED,
        EXCEPTION,
        VOID,
        RETURN_INITIATED,
    ];
}

/// Definition types – semantic category of the event.
pub mod definition_type {
    /// Key progression point in the shipment lifecycle; primary status indicator.
    pub const MILESTONE: &str = "MILESTONE";

    /// Informational update; supplementary detail without changing primary status.
    pub const INFO: &str = "INFO";

    /// Incident, error, or abnormal condition requiring attention.
    pub const EXCEPTION: &str = "EXCEPTION";

    /// All valid definition types.
    pub const ALL: &[&str] = &[MILESTONE, INFO, EXCEPTION];
}

/// Code scheme identifiers for codified meanings (external standards).
pub mod code_scheme {
    /// openPEPPOL TransportationStatusCode (PEPPOL-INCUBATION-LOGISTICS v3.0).
    pub const PEPPOL_TRANSPORTATION_STATUS: &str = "PEPPOL-TRANSPORTATION-STATUS";

    /// X12 EDI Element 157 – Shipment Status Code.
    pub const X12_157: &str = "X12-157";

    /// ISO/IEC 19987 – EPCIS event types.
    pub const EPCIS_EVENT: &str = "EPCIS-EVENT";
}

/// Check if a string is a valid harmonized event type.
#[inline]
pub fn is_valid_harmonized(s: &str) -> bool {
    harmonized::ALL.iter().any(|&v| v == s)
}

/// Check if a string is a valid definition type.
#[inline]
pub fn is_valid_definition_type(s: &str) -> bool {
    definition_type::ALL.iter().any(|&v| v == s)
}
