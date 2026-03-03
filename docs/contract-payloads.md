# Contract Payloads

Exact currently accepted payloads and input parameters for each contract that receives user-supplied or cross-contract data.

---

## 1. Service Contract

### init

| Parameter | Type | Validation |
|-----------|------|------------|
| `default_price` | `BigUint` | — |
| `forwarder_default_fee` | `OptionalValue<BigUint>` | If None, defaults to 0 |
| `capabilities_config` | `OptionalValue<ManagedBuffer>` | Optional; opaque blob (e.g. JSON) for routes, addons, serviceId; stored and returned by getCapabilities |

**Reference:** [schemas/config/service-config.schema.json](../schemas/config/service-config.schema.json) – for integration guidance; contract does not validate structure.

### getCapabilities (view)

Returns `MultiValue4<BigUint, BigUint, ManagedAddress, ManagedBuffer>`:
1. `default_price`
2. `forwarder_default_fee`
3. `carrier_address`
4. `capabilities_config` – opaque blob set at init; empty buffer if not set

### payload (validate, quote, routeKey)

| Attribute | Value |
|-----------|-------|
| **Type** | `ManagedBuffer` |
| **Used in** | `validate(payload, forwarder_agreement_addr?)`, `quote(payload, forwarder_agreement_addr?)`, `routeKey(payload)` |
| **Source** | Shipment.createShipment → passed to Service |

**On-chain validation:**
- `!payload.is_empty()` – empty payload rejected

**Semantic format:** Arbitrary bytes; carrier-specific. The contract does not parse or validate structure. It uses `keccak256(payload)` for quote_hash and route key generation.

**Reference schema:** [schemas/shipment-booking.schema.json](../schemas/shipment-booking.schema.json) – for integration guidance only; contract does not enforce JSON structure.

---

## 2. Shipment Contract

### shipment_payload (createShipment)

| Attribute | Value |
|-----------|-------|
| **Type** | `ManagedBuffer` |
| **Used in** | `createShipment(agreement_addr, service_id, shipment_payload, parcel_ids..., forwarder?, encrypted?)` |

**Validation:** Forwarded to Service.validate/quote; must pass Service validation (non-empty).

**Reference:** [schemas/shipment-booking.schema.json](../schemas/shipment-booking.schema.json)

### voidShipment

| Parameter | Type | Validation |
|-----------|------|------------|
| `tracking_number` | `ManagedBuffer` | Must exist; caller must be shipment_owner; must not be voided |

**Effect:** Sets shipment as voided; registers VOID event in Tracker. Refund (recapture cost) requires separate call to Agreement.refundForVoidedShipment by carrier, only when shipment has not been dispatched.

### refundForVoidedShipment (Agreement)

| Parameter | Type | Validation |
|-----------|------|------------|
| `tracking_number` | `ManagedBuffer` | Must have VOID event in Tracker; must not have DISPATCHED event |
| EGLD payment | `BigUint` | Must equal captured_for_shipment(tracking_number) |

**Caller:** Carrier only. Adds payment to customer deposit_balance. Requires Agreement.setTrackerContract to have been set.

### refundForReturnShipment (Agreement, payable)

| Parameter | Type | Validation |
|-----------|------|------------|
| `outbound_tracking_number` | `ManagedBuffer` | Must have a return with reimburse_shipper=true (Tracker.hasReimbursableReturn) |
| EGLD payment | `BigUint` | Must equal captured_for_shipment(outbound_tracking_number) |

**Caller:** Carrier only. Adds payment to deposit_balance; clears captured_for_shipment. Use when return was due to exception (damaged, lost, customs refused), not failure to collect.

### encrypted_payload (createShipment)

| Attribute | Value |
|-----------|-------|
| **Type** | `OptionalValue<ManagedBuffer>` |
| **Used in** | `createShipment(..., encrypted_payload?)` |
| **Storage** | Stored in Tracker; retrievable via `getEncryptedPayload(tracking_number)` |

**Format:** Opaque ciphertext. No on-chain format validation. Decrypt off-chain with `KDF(tracking_number, receiver_secret)`.

### createReturnShipment (carrier only)

| Parameter | Type | Validation |
|-----------|------|------------|
| `outbound_tracking_number` | `ManagedBuffer` | Must exist; not voided; must have DISPATCHED event; must not have DELIVERED event |
| `agreement_addr` | `ManagedAddress` | Non-zero; must have Agreement; caller must be carrier_address |
| `service_id` | `ManagedBuffer` | Must be in enabled_services (e.g. "RETURN") |
| `shipment_payload` | `ManagedBuffer` | Passed to Service.validate/quote |
| `parcel_ids` | `MultiValueEncoded<ManagedBuffer>` | Each must exist and belong to shipment owner |
| `reimburse_shipper` | `bool` | true when return due to exception (damaged, lost, customs refused); false when failure to collect |
| `encrypted_payload` | `OptionalValue<ManagedBuffer>` | Optional; stored in Tracker for return shipment |

**Effect:** Creates return shipment with tracking prefix "R"; registers BOOKED; emits RETURN_INITIATED on outbound. No charge to shipper. When `reimburse_shipper=true`, carrier may later call Agreement.refundForReturnShipment(outbound_tn) with EGLD.

---

## 3. Parcel Contract

### register_parcel parameters

| Parameter | Type | Validation |
|-----------|------|------------|
| `parcel_id` | `ManagedBuffer` | — |
| `reference` | `ManagedBuffer` | — |
| `description` | `ManagedBuffer` | — |
| `weight` | `u64` | Minimum 0; config-driven limits when parsed |
| `weight_unit` | `ManagedBuffer` | Must be one of: `G`, `KG`, `LB`, `OZ` |
| `item_ids` | `MultiValueEncoded<ManagedBuffer>` | If provided, minItems 1 |
| `serial` | `OptionalValue<ManagedBuffer>` | If empty, generated via Serial (prefix "P" + counter) |
| `dg_net_quantity` | `u64` | If > 0, adds one dangerous goods entry |
| `dg_net_quantity_unit` | `ManagedBuffer` | For dangerous goods |
| `dg_type_code` | `ManagedBuffer` | For dangerous goods |
| `dg_quantity` | `u32` | For dangerous goods |

**Entity rules (parcel.schema.json aligned):**
- `dangerousGoods`: maxItems 1
- `weightUnit` enum: G, KG, LB, OZ

**Reference:** [schemas/parcel.schema.json](../schemas/parcel.schema.json)

**Config-driven validation:** [schemas/config/parcel-config.schema.json](../schemas/config/parcel-config.schema.json) – weight limits when config parsing is implemented.

---

## 4. Tracker Contract

### registerEvent

| Parameter | Type | Validation |
|-----------|------|------------|
| `tracking_number` | `ManagedBuffer` | Non-empty |
| `event_type` | `ManagedBuffer` | Non-empty |
| `timestamp` | `u64` | — |
| `location` | `OptionalValue<ManagedBuffer>` | Optional; freeform |

**Harmonized event types** (recommended for display): `BOOKED`, `DISPATCHED`, `IN_TRANSIT`, `OUT_FOR_DELIVERY`, `DELIVERED`, `EXCEPTION`, `VOID`, `RETURN_INITIATED`. Carrier-specific codified codes may be stored and resolved to harmonized type off-chain.

**Reference:** [schemas/tracking-event-codes.schema.json](../schemas/tracking-event-codes.schema.json), [crates/dship-common/src/tracking_events.rs](../crates/dship-common/src/tracking_events.rs)

### registerShipment (called by Shipment only)

| Parameter | Type | Validation |
|-----------|------|------------|
| `tracking_number` | `ManagedBuffer` | Non-empty |
| `agreement_addr` | `OptionalValue<ManagedAddress>` | — |
| `forwarder_agreement_addr` | `OptionalValue<ManagedAddress>` | — |
| `encrypted_payload` | `OptionalValue<ManagedBuffer>` | Opaque ciphertext |

### registerReturnShipment (called by Shipment only)

| Parameter | Type | Validation |
|-----------|------|------------|
| `return_tracking_number` | `ManagedBuffer` | Non-empty |
| `outbound_tracking_number` | `ManagedBuffer` | The original shipment being returned |
| `agreement_addr` | `ManagedAddress` | Non-zero |
| `reimburse_shipper` | `bool` | true when return due to exception; false when failure to collect |
| `encrypted_payload` | `OptionalValue<ManagedBuffer>` | Optional; opaque ciphertext |

---

## 5. Onboarding Contract

### deployAgreement

| Parameter | Type | Validation |
|-----------|------|------------|
| `customer_owner` | `ManagedAddress` | — |
| `customer_pubkey` | `ManagedBuffer` | Ed25519 public key; used for signature verification |
| `agreement_config_hash` | `ManagedBuffer` | Hash of agreement config |
| `shipment_contract` | `ManagedAddress` | Non-zero |
| `expiry` | `u64` | `block_timestamp <= expiry` |
| `nonce` | `u64` | Must not be already used |
| `credit_limit` | `BigUint` | — |
| `enabled_services` | `MultiValueEncoded<ManagedBuffer>` | — |
| `signature` | `ManagedBuffer` | Ed25519 signature over hashed approval message |

**Approval message format:**  
`APPROVE_DEPLOY|{carrier_addr}|{factory_addr}|{code_hash}|{agreement_config_hash}|{expiry_8be}{nonce_8be}`  
(expiry and nonce as 8-byte big-endian, no delimiter between them)

### deployForwarderAgreement

| Parameter | Type | Validation |
|-----------|------|------------|
| `forwarder_owner` | `ManagedAddress` | — |
| `forwarder_pubkey` | `ManagedBuffer` | Ed25519 public key |
| `agreement_config_hash` | `ManagedBuffer` | — |
| `shipment_contract` | `ManagedAddress` | Non-zero |
| `expiry` | `u64` | `block_timestamp <= expiry` |
| `nonce` | `u64` | Must not be already used (per forwarder_owner) |
| `enabled_services` | `MultiValueEncoded<ManagedBuffer>` | — |
| `signature` | `ManagedBuffer` | Ed25519 signature |

**Approval message format:**  
`APPROVE_FORWARDER_DEPLOY|{carrier_addr}|{factory_addr}|{code_hash}|{agreement_config_hash}|{expiry_8be}{nonce_8be}`

**Reference:** [crates/dship-common/src/agreement.rs](../crates/dship-common/src/agreement.rs)

---

## 6. Agreement Contract

### authorizeShipment (internal – called by Shipment)

| Parameter | Type | Description |
|-----------|------|-------------|
| `service_id` | `ManagedBuffer` | Must be in enabled_services |
| `normalized_metrics` | `ManagedBuffer` | Passed from Service.quote; not validated structurally |
| `amount` | `BigUint` | Must be > 0; must be ≤ available (deposit − reserved + credit) |
| `quote_hash` | `ManagedBuffer` | keccak256(payload) from Service.quote |

No direct user payload; all values originate from Shipment/Service flow.

---

## 7. Serial Contract

### generate(prefix)

| Parameter | Type | Validation |
|-----------|------|------------|
| `prefix` | `OptionalValue<ManagedBuffer>` | Optional; if non-empty, prepended to counter |

**Output format:** `{prefix}{counter}` (e.g. prefix `"S"` → `"S12345"`, prefix `"P"` → `"P67890"`)

**Source:** Shipment (prefix "S") and Parcel (prefix "P") when serial omitted.

---

## 8. Pickup Contract

### requestPickup parameters

| Parameter | Type | Validation |
|-----------|------|------------|
| `tracking_numbers` | `MultiValueEncoded<ManagedBuffer>` | Non-empty; max 50 per batch |
| `agreement_addr` | `ManagedAddress` | Non-zero; must match agreement for each shipment |
| `slot_date` | `u64` | Unix date (e.g. start of day) |
| `slot_time_from` | `ManagedBuffer` | e.g. "09:00" |
| `slot_time_to` | `ManagedBuffer` | e.g. "17:00" |
| `location` | `OptionalValue<ManagedBuffer>` | Pickup address/location reference |

**Fee:** Agreement.getPickupFee if set; else Pickup.pickup_default_fee.

**Reference:** [schemas/agreement-config.schema.json](../schemas/agreement-config.schema.json) (availablePickupSlots, pickupFee)
