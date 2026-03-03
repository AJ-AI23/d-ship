# @dship/carrier-sdk

TypeScript SDK for d-ship carrier backend integrations. Enables carrier-only operations: onboarding (deploy agreements, forwarder agreements), service management (register forwarders), shipment admin (register services, create returns), tracker (register events), and agreement refunds.

## Installation

```bash
npm install @dship/carrier-sdk @multiversx/sdk-core
# For signing: npm install @multiversx/sdk-wallet
```

## Configuration

```ts
import { DshipCarrierClient } from "@dship/carrier-sdk";

const config = {
  network: "devnet" as const, // or "mainnet" | "testnet"
  apiUrl: "https://devnet-api.multiversx.com", // optional override
  senderAddress: "erd1...", // carrier wallet address (signs transactions)
  addresses: {
    onboarding: "erd1...",
    shipment: "erd1...",
    tracker: "erd1...",
    pickup: "erd1...",
    service: "erd1...",
  },
};

const client = new DshipCarrierClient(config);
```

## Usage

All write operations require a signer. The signer receives a `Transaction` and returns the signed transaction:

```ts
import { UserSecretKey } from "@multiversx/sdk-wallet";

const secretKey = UserSecretKey.fromMnemonic("your mnemonic ...");
const signer = async (tx) => {
  const signature = secretKey.sign(tx.serializeForSigning());
  return tx.applySignature(signature);
};
```

### Onboarding

Deploy customer and forwarder agreements:

```ts
// Deploy customer Agreement (requires pre-signed approval from customer)
const txHash = await client.onboarding.deployAgreement(signer, {
  customerOwner: "erd1...",
  customerPubkey: Buffer.from(/* 32-byte Ed25519 public key */),
  agreementConfigHash: "hex or buffer",
  shipmentContract: config.addresses.shipment,
  expiry: Math.floor(Date.now() / 1000) + 3600,
  nonce: 1,
  creditLimit: 0n,
  enabledServices: ["STANDARD", "EXPRESS"],
  signature: Buffer.from(/* Ed25519 signature */),
});

// Deploy ForwarderAgreement
await client.onboarding.deployForwarderAgreement(signer, {
  forwarderOwner: "erd1...",
  forwarderPubkey: Buffer.from(/* 32 bytes */),
  agreementConfigHash: "hex",
  shipmentContract: config.addresses.shipment,
  expiry,
  nonce,
  enabledServices: ["PUDO", "LAST_MILE"],
  signature: Buffer.from(/* ... */),
});

// Configure forwarder template (one-time setup)
await client.onboarding.setForwarderAgreementTemplate(signer, templateAddr);
await client.onboarding.setAllowedForwarderCodeHash(signer, codeHash);
```

### Service

Register and unregister forwarders:

```ts
await client.service.setCarrierAddress(signer, carrierAddr);
await client.service.registerForwarder(signer, forwarderAgreementAddr);
await client.service.unregisterForwarder(signer, forwarderAgreementAddr);

// Views (no signer)
const caps = await client.service.getCapabilities();
const count = await client.service.getRegisteredForwarderCount();
const forwarders = await client.service.getRegisteredForwarders(0, 50);
```

### Shipment

Carrier admin and return shipments:

```ts
await client.shipment.registerService(signer, "STANDARD", serviceAddr);
await client.shipment.setParcelContract(signer, parcelAddr);
await client.shipment.setPickupContract(signer, pickupAddr);

// Create return shipment (carrier only)
await client.shipment.createReturnShipment(signer, {
  outboundTrackingNumber: "S12345",
  agreementAddr: "erd1...",
  serviceId: "RETURN",
  payload: JSON.stringify({ /* return booking */ }),
  parcelIds: ["P001"],
  reimburseShipper: true, // true when exception; false when failure to collect
  encryptedPayload: undefined, // optional
});

// Views
const exists = await client.shipment.hasShipment("S12345");
const voided = await client.shipment.isVoided("S12345");
const outbound = await client.shipment.getOutboundForReturn("R67890");
```

### Tracker

Register events and configure contracts:

```ts
await client.tracker.setShipmentContract(signer, shipmentAddr);
await client.tracker.setPickupContract(signer, pickupAddr);

// Register tracking event
await client.tracker.registerEvent(
  signer,
  "S12345",
  "IN_TRANSIT",
  Math.floor(Date.now() / 1000),
  "Warehouse A"
);

// Views
const status = await client.tracker.getTrackingStatus("S12345");
const dispatched = await client.tracker.hasDispatched("S12345");
const agreement = await client.tracker.getAgreementForShipment("S12345");
const hasReimburse = await client.tracker.hasReimbursableReturn("S12345");
```

### Agreement (Carrier Refunds)

Refund voided or return shipments. The carrier sends EGLD back to the customer's agreement:

```ts
// Refund voided shipment (only when voided before DISPATCHED)
await client.agreement.refundForVoidedShipment(
  signer,
  agreementAddr,
  "S12345",
  capturedAmount // must match captured_for_shipment
);

// Refund for return (when reimburse_shipper was true)
await client.agreement.refundForReturnShipment(
  signer,
  agreementAddr,
  "S12345", // outbound tracking number
  capturedAmount
);

// Views (pass agreement address)
const owner = await client.agreement.getCustomerOwner(agreementAddr);
const balance = await client.agreement.getDepositBalance(agreementAddr);
```

## Reference

- [docs/contract-method-map.md](../../docs/contract-method-map.md) – method names and access control
- [docs/contract-payloads.md](../../docs/contract-payloads.md) – exact parameters
- [docs/contract-architecture-README.md](../../docs/contract-architecture-README.md) – architecture overview
