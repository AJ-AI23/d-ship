# d-ship Schemas

JSON schemas for d-ship contract data, aligned with the [nShift Go API](https://api.qa.nshiftgo.com/api-docs) (v1.4.26).

## Structure

### Data Schemas (nShift-aligned)

| Schema | Description | API Reference |
|--------|-------------|---------------|
| `address.schema.json` | Address (location, company, contact) | AddressRequest |
| `parcel.schema.json` | Parcel (weight, packing, items) | ParcelRequest |
| `parcel-item.schema.json` | Parcel item (product, quantity) | ParcelItemRequest |
| `party.schema.json` | Party (type, roles, address id) | Party |
| `shipment-booking.schema.json` | Shipment booking request | ShipmentBookingRequest |
| `tracker.schema.json` | Tracker (status, shipment, parcels) | TrackerRequest |
| `tracking-event.schema.json` | Single tracking event | Tracker status enum |
| `serial.schema.json` | Serial type config | SerialResponse |

### Config Schemas (d-app deployment)

| Schema | Contract | Used by |
|--------|----------|---------|
| `config/shipment-config.schema.json` | shipment | `create_shipment` validation |
| `config/parcel-config.schema.json` | parcel | `register_parcel` validation |
| `config/tracker-config.schema.json` | tracker | `register_event` validation |
| `config/serial-config.schema.json` | serial | `generate` format rules |

### Validation Behavior Tree (d-app config)

| Schema | Description |
|--------|-------------|
| `behavior-tree.schema.json` | Behavior3-style AST for input validation. Single root node, `{ id, type, children?, params? }`. Success/Failure only. Embeds in config via `validationTree`. Compiles to Rust at build time. Node types: Sequence, Selector, Inverter, RangeCheck, EnumCheck, RegexCheck, Condition, OracleCall, ProofRequired, GasLimit. |

See [docs/behavior-tree-README.md](../docs/behavior-tree-README.md) for compilation steps and UI implementation guide.

## Validation

Config passed to contract `init` at deployment should validate against the corresponding config schema:

```bash
# Example: validate shipment config
npx ajv validate -s schemas/config/shipment-config.schema.json -d config/shipment.example.json
```

## nShift API Mapping

- **Addresses** → `location` (streetName, postalCode, city, countryCode required)
- **Parties** → `type` ADDRESS\|PUDO, `roles` (SENDER, RECEIVER, etc.), `id` (address uuid)
- **Parcels** → weight at parcel/packing/item level; packing dimensions optional
- **Shipment Booking** → channel, parties, carrier/service defs, parcel ids
- **Tracker** → status BOOKED\|DISPATCHED\|VOID
