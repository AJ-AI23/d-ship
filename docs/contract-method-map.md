# Contract Method Map

A map of all contract methods and their interactions.

---

## 1. Contract Overview

| Contract | Purpose |
|----------|---------|
| **Onboarding** | Verifies customer/forwarder signatures; deploys Agreement and ForwarderAgreement contracts |
| **Agreement** | Per-customer agreement and billing; holds deposits, reserves, captures; supports split to forwarder |
| **ForwarderAgreement** | Carrier-forwarder agreement; forwarder receives payment via receivePayment (split capture) |
| **Service** | Service-specific validation and pricing; forwarder registry; validates forwarders on-chain; lookup via indexers off-chain |
| **Shipment** | Orchestrates shipment creation; validates agreement, parcels, forwarder on-chain; lookup via indexers off-chain |
| **Serial** | Generates unique tracking numbers (shipment and parcel level) |
| **Parcel** | Registers parcels, generates parcel serials via Serial, exposes label formats |
| **Tracker** | Registers tracking events; Carrier, Shipment, Forwarder, or Pickup may register; encrypted payload for receiver |
| **Pickup** | Schedules pickup after shipment creation; charges shipper via Agreement; registers DISPATCHED events |

---

## 2. Method Inventory

### Onboarding

| Method | Type | Access | Description |
|--------|------|--------|-------------|
| `init(agreement_template, allowed_code_hash)` | Endpoint | Deployer (carrier) | Initialize onboarding; caller becomes `carrier_address` |
| `deployAgreement(...)` | Endpoint | Carrier only | Verify Ed25519 signature, deploy new Agreement via `deploy_from_source_contract` |
| `setForwarderAgreementTemplate(template)` | Endpoint | Carrier only | Set ForwarderAgreement template for deployForwarderAgreement |
| `setAllowedForwarderCodeHash(code_hash)` | Endpoint | Carrier only | Set allowed code hash for ForwarderAgreement deployment |
| `deployForwarderAgreement(forwarder_owner, forwarder_pubkey, agreement_config_hash, shipment_contract, expiry, nonce, enabled_services, signature)` | Endpoint | Carrier only | Verify forwarder Ed25519 signature, deploy new ForwarderAgreement |
| `getDeployedAccounts` | View | Anyone | Returns all deployed agreement addresses |
| `getAgreementTemplate`, `getForwarderAgreementTemplate`, `getAllowedCodeHash`, `getAllowedForwarderCodeHash`, `getCarrierAddress` | View | Anyone | Storage getters |

### ForwarderAgreement

| Method | Type | Access | Description |
|--------|------|--------|-------------|
| `init(forwarder_owner, carrier_address, carrier_shipment_contract, agreement_config_hash, enabled_services)` | Endpoint | On deploy | Set forwarder, carrier, shipment contract, config; no deposit/credit |
| `receivePayment` | Endpoint (payable) | Agreement (via split capture) | Receive EGLD; forwards to `forwarder_owner` |
| `getForwarderOwner`, `getCarrierAddress`, `getCarrierShipmentContract`, `getAgreementConfigHash`, `getEnabledServices` | View | Anyone | Storage getters |

### Agreement

| Method | Type | Access | Description |
|--------|------|--------|-------------|
| `init(...)` | Endpoint | On deploy | Set customer, carrier, shipment contract, config, credit limit |
| `deposit` | Endpoint (payable) | Anyone | Add EGLD to customer deposit balance |
| `authorizeShipment(service_id, normalized_metrics, amount, quote_hash)` | View | Called by Shipment | Check service enabled, amount ≤ available (deposit − reserved + credit) |
| `authorizePickup(amount)` | View | Called by Pickup | Check amount ≤ available balance |
| `setPickupContract(addr)`, `setPickupFee(amount)`, `setPickupSlotsHash(hash)`, `setTrackerContract(addr)` | Endpoint | Shipment only | Configure pickup and tracker for this agreement |
| `reserve(amount, reference)` | Endpoint | Shipment or Pickup | Reserve funds; returns `reservation_id` |
| `capture(reservation_id, forwarder_agreement_addr?, forwarder_amount?)` | Endpoint | Shipment or Pickup | Capture reserved funds; validates ForwarderAgreement exists; transfer to carrier; optionally split |
| `release(reservation_id)` | Endpoint | Shipment or Pickup | Release reserved funds (no transfer) |
| `refundForVoidedShipment(tracking_number)` | Endpoint (payable) | Carrier only | Refund captured shipment cost to customer; requires VOID event, no DISPATCHED |
| `refundForReturnShipment(outbound_tracking_number)` | Endpoint (payable) | Carrier only | Refund outbound cost when return due to exception; requires reimbursable return |
| `getReservation(reservation_id)` | View | Anyone | Get reservation state |
| `getCustomerOwner`, `getCarrierAddress`, etc. | View | Anyone | Storage getters |

### Service

| Method | Type | Access | Description |
|--------|------|--------|-------------|
| `init(default_price, forwarder_default_fee?, capabilities_config?)` | Endpoint | Deployer | Set default price; optional forwarder fee (default 0); optional capabilities blob (routes, addons, etc.) |
| `setCarrierAddress(carrier)` | Endpoint | Carrier only | Set carrier for forwarder registration |
| `registerForwarder(forwarder_agreement_addr)` | Endpoint | Carrier only | Validate ForwarderAgreement exists on-chain, add to registry |
| `unregisterForwarder(addr)` | Endpoint | Carrier only | Remove forwarder from registry |
| `validate(payload, forwarder_agreement_addr?)` | View | Anyone | Validate payload; if forwarder provided, verify in registry |
| `quote(payload, forwarder_agreement_addr?)` | View | Anyone | Return `(normalized_metrics, amount, quote_hash, forwarder_addr, forwarder_amount)`; adds forwarder fee when registered |
| `routeKey(payload)` | Endpoint | Anyone | Route key for restrictions; stub: keccak256(payload) |
| `getRegisteredForwarderCount` | View | Anyone | Total forwarders; for indexer bootstrap |
| `getRegisteredForwarders(offset, limit)` | View | Anyone | Paginated list; lookup done off-chain via indexers |
| `getCapabilities` | View | Anyone | Returns (default_price, forwarder_default_fee, carrier_address, capabilities_config); single call for full deploy-time config |
| `getDefaultPrice`, `getForwarderDefaultFee` | View | Anyone | Storage getters |

### Shipment

| Method | Type | Access | Description |
|--------|------|--------|-------------|
| `init(allowed_factory, serial_contract, tracker_contract?, parcel_contract?)` | Endpoint | Deployer | Set allowed factory, Serial, optional Tracker, optional Parcel |
| `registerService(service_id, service_addr)` | Endpoint | Anyone* | Map service_id → Service contract |
| `setParcelContract(parcel_contract)` | Endpoint | Anyone* | Set Parcel contract for parcel existence validation |
| `setPickupContract(pickup_contract)` | Endpoint | Anyone* | Set Pickup contract address |
| `getServiceAddress(service_id)` | View | Anyone | Service address for indexer; returns None if not registered |
| `hasShipment(tracking_number)` | View | Anyone | Returns true if shipment exists; used by Pickup for validation |
| `isVoided(tracking_number)` | View | Anyone | Returns true if shipment has been voided; used by Pickup and off-chain consumers |
| `createShipment(agreement_addr, service_id, payload, parcel_ids..., forwarder_agreement_addr?, encrypted_payload?)` | Endpoint | Anyone | Full flow; validates agreement, parcels on-chain; optional forwarder; optional encrypted_payload |
| `createReturnShipment(outbound_tn, agreement_addr, service_id, payload, parcel_ids..., reimburse_shipper, encrypted_payload?)` | Endpoint | Carrier only | Create return; no charge; reimbursement via Agreement.refundForReturnShipment when reimburse_shipper |
| `voidShipment(tracking_number)` | Endpoint | Shipment owner only | Void shipment; registers VOID event in Tracker; refund via Agreement.refundForVoidedShipment when not yet dispatched |
| `getOutboundForReturn(tracking_number)` | View | Anyone | Returns outbound tracking number for a return; None if not a return |

*Access to `registerService` is not restricted in the current implementation.

### Serial

| Method | Type | Access | Description |
|--------|------|--------|-------------|
| `init(config)` | Endpoint | Deployer | Initialize with format config |
| `generate(prefix)` | Endpoint | Anyone | Generate unique serial: prefix + counter (e.g. "S12345", "P67890") |
| `getConfig` | View | Anyone | Config buffer |

### Parcel

| Method | Type | Access | Description |
|--------|------|--------|-------------|
| `init(config, serial_contract, barcode_format?)` | Endpoint | Deployer | Initialize; optional barcode_format defaults to CODE128 |
| `upgrade(config, serial_contract?)` | Endpoint | Deployer | Upgrade config; optionally set serial_contract |
| `register_parcel(..., serial?)` | Endpoint | Anyone | Register parcel; serial optional—if omitted, generated via Serial |
| `hasParcel(parcel_id)` | View | Anyone | Returns true if parcel exists on-chain; used by Shipment for validation |
| `getLabelFormat(parcel_id)` | View | Anyone | Returns (barcode_format, serial) for off-chain label generation |
| `getConfigHash` | View | Anyone | Config buffer |

### Tracker

| Method | Type | Access | Description |
|--------|------|--------|-------------|
| `init(config)` | Endpoint | Deployer | Initialize; caller becomes carrier |
| `setShipmentContract(shipment_contract)` | Endpoint | Carrier only | Allow Shipment to call registerShipment |
| `setPickupContract(pickup_contract)` | Endpoint | Carrier only | Allow Pickup to call registerEvent |
| `registerEvent(tracking_number, event_type, timestamp, location?)` | Endpoint | Carrier, Shipment, Forwarder, or Pickup | Store tracking event |
| `hasDispatched(tracking_number)` | View | Anyone | Returns true if shipment has DISPATCHED event |
| `hasEventType(tracking_number, event_type)` | View | Anyone | Returns true if shipment has event of given type |
| `getAgreementForShipment(tracking_number)` | View | Anyone | Returns agreement address for shipment |
| `registerShipment(tracking_number, agreement_addr?, forwarder_agreement_addr?, encrypted_payload?)` | Endpoint | Shipment only | Initial BOOKED event; optional agreement, forwarder, encrypted payload |
| `registerReturnShipment(return_tn, outbound_tn, agreement_addr, reimburse_shipper, encrypted_payload?)` | Endpoint | Shipment only | Register return shipment; sets reimbursable flag for Agreement refund |
| `hasReimbursableReturn(outbound_tracking_number)` | View | Anyone | Returns true if outbound has return with reimburse_shipper |
| `getTrackingStatus(tracking_number)` | View | Anyone | Returns plaintext events (event_type, timestamp, location) |
| `getEncryptedPayload(tracking_number)` | View | Anyone | Returns ciphertext; caller decrypts off-chain |

### Pickup

| Method | Type | Access | Description |
|--------|------|--------|-------------|
| `init(shipment_contract, tracker_contract, pickup_default_fee?)` | Endpoint | Deployer | Set shipment, tracker; caller becomes carrier; optional default fee |
| `requestPickup(tracking_numbers..., agreement_addr, slot_date, slot_time_from, slot_time_to, location?)` | Endpoint | Anyone | Validate shipments (reject voided), charge via Agreement, register DISPATCHED events |
| `getShipmentContract`, `getTrackerContract`, `getCarrierAddress`, `getPickupDefaultFee` | View | Anyone | Storage getters |

---

## 3. Cross-Contract Interaction Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            EXTERNAL ACTORS                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  Carrier          Customer         Anyone (for deposits, parcels, labels)     │
└──────┬────────────────┬────────────────┬───────────────────────────────────┘
       │                 │                │
       │ deployAgreement  │ deposit        │ deposit
       ▼                 │                │
┌──────────────┐         │                │
│  ONBOARDING  │         │                │
│              │ deploy_from_source_contract                                  │
│  - deployAgreement ──────────────────────► creates new AGREEMENT instance    │
└──────────────┘         │                │
                         │                │
                         ▼                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  AGREEMENT                                                                   │
│  - deposit() ◄────────────────────────────────────────────────────────────  │
│  - authorize_shipment() ◄─── only via Shipment proxy                         │
│  - authorizePickup() ◄─── via Pickup proxy                                   │
│  - reserve() ◄────────────── via Shipment or Pickup proxy                    │
│  - capture() ◄────────────── via Shipment or Pickup proxy                     │
│  - release() ◄────────────── via Shipment or Pickup proxy                     │
└─────────────────────────────────────────────────────────────────────────────┘
       ▲
       │
       │ createShipment(agreement_addr, ..., parcel_ids)
       │
┌──────┴──────┐
│  SHIPMENT   │
│             │
│  createShipment flow:                                                         │
│    0. Agreement.carrier_shipment_contract == self (validate)                   │
│    1. Parcel.hasParcel(parcel_id) for each (validate)                         │
│    2. serial_proxy.generate("S")        ──────►  SERIAL.generate (tracking)   │
│    3. service_proxy.validate(payload, forwarder?)  ──►  SERVICE.validate     │
│    4. service_proxy.quote(payload, forwarder?)     ──►  SERVICE.quote         │
│    5. agreement_proxy.authorize_shipment ────►  AGREEMENT.authorize_shipment  │
│    6. agreement_proxy.reserve          ──────►  AGREEMENT.reserve           │
│    7. (store shipment + parcel_ids + forwarder_for_shipment)                 │
│    8. agreement_proxy.capture(res_id, fwd?, amt?)  ──►  AGREEMENT.capture     │
│       └─ validates ForwarderAgreement exists; sends fwd_amount ──►  receivePayment │
│    9. tracker_proxy.registerShipment(..., fwd?)  ──►  TRACKER (optional)    │
└──────┬──────┘
       │
       │ service_registry → Service; serial_contract → Serial; tracker_contract → Tracker
       │
       │ requestPickup(tracking_numbers, agreement_addr, slot...) [after shipment created]
       ▼
┌──────────────┐
│  PICKUP      │  requestPickup flow:
│              │    1. Shipment.hasShipment(tn) for each (validate)
│              │    2. Shipment.isVoided(tn) for each (reject voided)
│              │    3. Tracker.getAgreementForShipment(tn) == agreement_addr (validate)
│              │    4. Agreement.authorizePickup(amount)
│              │    5. Agreement.reserve(amount, reference)
│              │    6. Tracker.registerEvent(tn, DISPATCHED) for each
│              │    7. Agreement.capture(reservation_id)
└──────────────┘
┌──────────────┐         ┌──────────────┐
│  SERVICE     │         │  SERIAL      │
│  - validate  │         │  - generate  │
│  - quote     │         └──────▲───────┘
└──────────────┘                │
                               │ register_parcel(serial?)
┌──────────────┐                │
│  PARCEL      │◄───────────────┘
│  - register_parcel (calls Serial.generate("P") when serial omitted)
│  - getLabelFormat(parcel_id) ◄──── User (post-creation label retrieval)
└──────────────┘

┌──────────────┐
│  TRACKER     │◄─── Shipment.registerShipment (BOOKED); Carrier/Shipment/Forwarder/Pickup.registerEvent
│  - getTrackingStatus(tracking_number) ◄──── Anyone (plaintext status)
│  - getEncryptedPayload(tracking_number) ◄──── Anyone (ciphertext; decrypt off-chain)
└──────────────┘
```

---

## 4. createShipment Flow (Detailed)

```
User calls Shipment.createShipment(agreement_addr, service_id, payload, parcel_ids..., forwarder?, encrypted?)
    │
    ├─► Agreement.carrierShipmentContract == self [validate agreement for this Shipment]
    │
    ├─► Parcel.hasParcel(parcel_id) for each    [validate parcels exist when parcel_contract set]
    │
    ├─► Serial.generate(prefix "S")         [execute_on_dest_context]
    │       └─ returns tracking_number
    │
    ├─► Service.validate(payload, forwarder?)    [execute_on_dest_context]
    │       └─ returns bool; if forwarder, verifies in registry
    │
    ├─► Service.quote(payload, forwarder?)       [execute_on_dest_context]
    │       └─ returns (normalized_metrics, amount, quote_hash, fwd_addr, fwd_amount)
    │
    ├─► Agreement.authorizeShipment(service_id, normalized_metrics, amount, quote_hash)
    │       └─ returns bool (checks enabled services, available balance)
    │
    ├─► Agreement.reserve(amount, tracking_number)
    │       └─ returns reservation_id
    │
    ├─► (Store shipment + parcel_ids + forwarder_for_shipment in Shipment contract)
    │
    ├─► Agreement.capture(reservation_id, forwarder?, forwarder_amount?)
    │       └─ Validates ForwarderAgreement exists when forwarder provided
    │       └─ Transfers (amount - forwarder_amount) to carrier
    │       └─ When forwarder: direct_egld to ForwarderAgreement.receivePayment
    │
    └─► Tracker.registerShipment(tracking_number, agreement?, forwarder?, encrypted?)
            └─ Stores forwarder_for_shipment so forwarder can registerEvent
```

## 4b. requestPickup Flow (Post-Shipment)

```
User calls Pickup.requestPickup(tracking_numbers[], agreement_addr, slot_date, slot_time_from, slot_time_to, location?)
    │
    ├─► Shipment.hasShipment(tn) for each [validate shipment exists]
    ├─► Shipment.isVoided(tn) for each [reject voided shipments]
    ├─► Tracker.getAgreementForShipment(tn) == agreement_addr [validate same agreement]
    ├─► Agreement.getPickupContract == self [validate Pickup authorized]
    ├─► Fee: Agreement.getPickupFee or Pickup.pickup_default_fee
    ├─► Agreement.authorizePickup(amount)
    ├─► Agreement.reserve(amount, reference)
    ├─► Tracker.registerEvent(tn, "DISPATCHED", timestamp, location) for each
    └─► Agreement.capture(reservation_id)
```

## 4b2. Void Shipment Flow (with optional Refund)

```
Shipper calls Shipment.voidShipment(tracking_number)
    │
    ├─► require caller == shipment_owner
    ├─► require shipment exists, not already voided
    ├─► Set shipment_voided(tn) = true
    └─► Tracker.registerEvent(tn, "VOID", timestamp, None)

Refund (only when voided before DISPATCHED):
    Carrier calls Agreement.refundForVoidedShipment(tracking_number) with EGLD = captured amount
    │
    ├─► require caller == carrier_address
    ├─► require payment == captured_for_shipment(tn)
    ├─► require Tracker.hasEventType(tn, "VOID")
    ├─► require !Tracker.hasDispatched(tn)
    └─► Add payment to deposit_balance; clear captured_for_shipment

Setup: Agreement.setTrackerContract(addr) must be called by Shipment (or carrier) once per Agreement.
```

## 4b3. Return Shipment Flow (Carrier-Initiated)

```
Carrier calls Shipment.createReturnShipment(outbound_tn, agreement_addr, service_id, payload, parcel_ids..., reimburse_shipper, encrypted?)
    │
    ├─► require outbound exists, not voided, has DISPATCHED
    ├─► require caller == Agreement.carrier_address
    ├─► Serial.generate(prefix "R") → return tracking number
    ├─► Validate parcels, service
    ├─► Store return record; outbound_for_return, return_reimburse_shipper, return_shipments
    ├─► Tracker.registerReturnShipment(return_tn, outbound_tn, agreement_addr, reimburse_shipper, encrypted?)
    └─► Tracker.registerEvent(outbound_tn, "RETURN_INITIATED", timestamp, None)

Reimbursement (when reimburse_shipper = true, i.e. exception during delivery):
    Carrier calls Agreement.refundForReturnShipment(outbound_tracking_number) with EGLD = captured amount
    │
    ├─► require caller == carrier_address
    ├─► require payment == captured_for_shipment(outbound_tn)
    ├─► require Tracker.hasReimbursableReturn(outbound_tn)
    └─► Add payment to deposit_balance; clear captured_for_shipment

Note: reimburse_shipper = false when return due to failure to collect (no reimbursement).
```

## 4c. Parcel Registration & Label Retrieval Flow

```
Pre-creation: User registers parcels
    Parcel.register_parcel(..., serial: OptionalValue::None)
        └─► Parcel calls Serial.generate(prefix "P") for parcel-level serial
        └─► Stores parcel with generated serial

Post-creation: User retrieves labels for printing
    Parcel.getLabelFormat(parcel_id)  [view]
        └─► returns (barcode_format, serial) for each parcel in shipment
```

---

## 5. Deployment Flow

### Customer Agreement
```
Carrier calls Onboarding.deployAgreement(...)
    │
    ├─ Verify caller == carrier_address
    ├─ Verify timestamp ≤ expiry
    ├─ Verify nonce not used
    ├─ Build approval message (carrier, factory, code_hash, config_hash, expiry, nonce)
    ├─ keccak256(message) → msg_hash
    ├─ verify_ed25519(customer_pubkey, msg_hash, signature)
    │
    └─ deploy_from_source_contract(agreement_template, init_args...)
            └─ Creates new Agreement instance
            └─ Records in deployed_accounts
```

### Forwarder Agreement
```
Carrier must first: setForwarderAgreementTemplate(addr), setAllowedForwarderCodeHash(hash)

Carrier calls Onboarding.deployForwarderAgreement(...)
    │
    ├─ Verify caller == carrier_address
    ├─ Verify timestamp ≤ expiry
    ├─ Verify nonce not used (per forwarder_owner)
    ├─ Build message: APPROVE_FORWARDER_DEPLOY|carrier|factory|code_hash|config_hash|expiry|nonce
    ├─ keccak256(message) → msg_hash
    ├─ verify_ed25519(forwarder_pubkey, msg_hash, signature)
    │
    └─ deploy_from_source_contract(forwarder_template, init_args...)
            └─ Creates new ForwarderAgreement instance (no credit_limit)
            └─ Records in deployed_accounts
```

### Forwarder as Customer (Multi-Carrier)
When a forwarder books last-mile with another carrier: use standard `deployAgreement` (forwarder_owner as customer). Forwarder deposits and uses credit like any customer.

---

## 6. Trust & Access Summary

| From | To | Methods | Condition |
|------|----|---------|-----------|
| Onboarding | Agreement | `init` (via deploy) | Only when deploying; carrier is caller |
| Shipment | Serial | `generate` | Serial contract must be set in Shipment init |
| Shipment | Tracker | `registerShipment` | Tracker must be set; Tracker must have Shipment set via setShipmentContract |
| Shipment | Agreement | `carrier_shipment_contract`, `setPickupContract` | Validate agreement; configure pickup per agreement |
| Shipment | Parcel | `hasParcel` | Validate parcel_ids exist when parcel_contract set |
| Shipment | Service | `validate`, `quote` | Service must be in `service_registry` |
| Shipment | Agreement | `authorize_shipment`, `reserve`, `capture` | Agreement must list this Shipment as `carrier_shipment_contract` |
| Parcel | Serial | `generate` | Serial contract must be set in Parcel init; only when serial omitted |
| Pickup | Shipment | `hasShipment`, `isVoided` | Validate shipment exists, not voided |
| Pickup | Tracker | `getAgreementForShipment`, `registerEvent` | Validate agreement; register DISPATCHED |
| Pickup | Agreement | `authorizePickup`, `reserve`, `capture` | Charge shipper; only if Pickup is set in Agreement |
| Agreement | — | `reserve`, `capture`, `release` | Only if caller == `carrier_shipment_contract` or `pickup_contract` |
| Agreement | ForwarderAgreement | `receivePayment` (EGLD) | During capture when forwarder split; forwards to forwarder_owner |
| Agreement | Tracker | `hasDispatched`, `hasEventType` | Validate void + no dispatch for refundForVoidedShipment |
| Agreement | — | `deposit`, `refundForVoidedShipment` | Anyone (deposit); carrier (refund) |
| Onboarding | — | `deployAgreement`, `deployForwarderAgreement` | Only if caller == `carrier_address` |
| User | Parcel | `getLabelFormat` | After shipment + parcels exist; for label printing |
| User | Tracker | `getTrackingStatus`, `getEncryptedPayload` | Anyone with tracking number; decrypt payload off-chain with key |

---

## 7. Identifier Validation and Off-Chain Lookup

Contracts assume identifiers (agreement_addr, parcel_ids, forwarder_agreement_addr, service_id) are obtained from off-chain indexers. On-chain validation:

| Contract | Validates |
|----------|-----------|
| Shipment | Agreement.carrierShipmentContract == self; Parcel.hasParcel for each parcel_id |
| Service | ForwarderAgreement exists (getForwarderOwner) on registerForwarder |
| Agreement | ForwarderAgreement exists before capture split |
| Pickup | Shipment exists, not voided; agreement matches; Pickup authorized in Agreement |
| Onboarding | shipment_contract, template, code_hash non-zero |

Lookup and search (e.g. list forwarders for service, filter by PUDO) is done off-chain via indexers. On-chain views (getRegisteredForwarders pagination, getServiceAddress) support indexer bootstrap.

---

## 8. No Direct Cross-Contract Calls

| Contract | Does not call |
|----------|----------------|
| Agreement | Other contracts except ForwarderAgreement (receivePayment), Tracker (refund validation) |
| Service | ForwarderAgreement (getForwarderOwner on registerForwarder for existence validation) |
| Serial | Any other contract |
| Tracker | Agreement (billing deferred); does not call other contracts |
| Pickup | Shipment, Tracker, Agreement (requestPickup flow) |
| Onboarding | Agreement methods (only deploys new instances) |

---

## 9. Library Usage

| Contract | Uses |
|----------|------|
| Agreement | `dship_common::billing` (Reservation, ReservationState); forwarder_agreement proxy |
| Onboarding | `dship_common::agreement::{build_approval_message, build_forwarder_approval_message}` |
| Shipment | `dship_common::entities` (Shipment struct) |
| Parcel | `dship_common::{entities, ownership, validation}` |
| Tracker | `dship_common::entities::TrackingEventRecord` |
| Pickup | agreement, shipment, tracker proxies |
