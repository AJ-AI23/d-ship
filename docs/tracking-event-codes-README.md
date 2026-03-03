# Tracking Event Codes

Standard definitions for shipment tracking events. Composed of:

1. **Harmonized event type** – Simplified status for UI (BOOKED, DISPATCHED, DELIVERED, etc.)
2. **Definition type** – Semantic category: MILESTONE, INFO, or EXCEPTION
3. **Codified meaning** – Optional reference to external standards for machine interoperability

## Harmonized Event Types

| Code | Description |
|------|-------------|
| BOOKED | Shipment booked/created |
| DISPATCHED | Dispatched from origin |
| IN_TRANSIT | In transit |
| OUT_FOR_DELIVERY | Out for delivery |
| DELIVERED | Delivered |
| EXCEPTION | Incident, error, or abnormal condition |
| VOID | Cancelled/voided |

## Definition Types

| Code | Description |
|------|-------------|
| MILESTONE | Key progression point; primary status indicator |
| INFO | Informational update; supplementary detail |
| EXCEPTION | Incident, error, or abnormal condition requiring attention |

## ISO / Industry Compatibility

Definitions align with:

- **openPEPPOL TransportationStatusCode** (PEPPOL-INCUBATION-LOGISTICS v3.0) – [codelist](https://docs.peppol.eu/logistics/2024-Q1/codelist/TransportationStatusCode/)
- **X12 EDI Element 157** – Shipment Status Code
- **ISO/IEC 19987** – EPCIS event types

The catalog in `schemas/tracking-event-definitions.json` maps each harmonized type and definition type to codified meanings (e.g. PEPPOL code "21" = Delivery completed, "18" = Damaged in transit).

## Usage

**Rust (dship-common):**
```rust
use dship_common::tracking_events::{harmonized, definition_type, is_valid_harmonized};

// Use constants
let event = harmonized::DELIVERED;
let def_type = definition_type::MILESTONE;

// Validate
assert!(is_valid_harmonized("BOOKED"));
```

**On-chain:** The Tracker contract stores `event_type` as a string. Use harmonized types (e.g. "BOOKED") or, if the carrier system uses codified codes, store the code and resolve to harmonized type off-chain for display.
