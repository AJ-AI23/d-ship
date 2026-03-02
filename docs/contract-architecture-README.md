# 🚢 D-Ship Contract Architecture – Build Specification

This document defines the complete smart contract architecture for a carrier-driven, customer-approved shipment ecosystem on MultiversX.

---

# 1️⃣ Contracts To Build

---

## A. Onboarding (Deployed by Carrier)

Responsible for:

- Verifying customer approval signatures
- Deploying per-customer contracts
- Binding deployed contracts to:
  - Specific carrier shipment contract
  - Specific contract code hash
  - Specific agreement configuration hash
- Preventing replay attacks (nonce tracking)
- Emitting `AgreementDeployed` event

### Must Verify:

- Customer signature validity
- Agreement configuration hash
- Code hash matches approved contract template
- Expiry timestamp not exceeded
- Nonce unused

---

## B. Agreement Contract (Deployed Per Customer via Onboarding)

Represents the on-chain customer agreement and billing authority.

### Stores:

- `customerOwner`
- `carrierAddress`
- `carrierShipmentContract`
- `agreementConfigHash`
- Service configurations:
  - Enabled services
  - Route restrictions
  - Quotas
  - Pricing model reference
- Billing state:
  - Deposit balance
  - Credit limits
  - Reserved amounts
- Nonce counter
- Reservation records

### Exposes:

- `authorizeShipment(serviceId, normalizedMetrics, amount, quoteHash)`
- `reserve(amount, reference)`
- `capture(reservationId)`
- `release(reservationId)`
- `recordUsage(...)`
- `deposit()`

### Enforces:

- Only registered carrier shipment contract may charge
- Only allowed services and routes
- Quota enforcement
- Price validation
- Spending cap enforcement

---

## C. Shipment Contract (Deployed by Carrier)

Acts as the shipment creation dApp backend.

### Stores:

- Mapping of `serviceId → Service`
- Allowed customer contract template hash

### Exposes:

- `createShipment(customerContractAddr, serviceId, shipmentPayload)`

### Internal Flow:

1. Validate shipment payload
2. Call Service:
   - `validate(payload)`
   - `quote(payload)`
3. Call Agreement:
   - `authorizeShipment(...)`
   - `reserve(amount)`
4. Create shipment entity (NFT or stored record)
5. Call `capture(reservationId)`
6. Emit `ShipmentCreated`

If any step fails:
- Revert transaction OR
- Release reservation

---

## D. Service (Optional Per Service)

Encapsulates service-specific validation and pricing logic.

### Exposes:

- `validate(payload)`
- `quote(payload)`
- `routeKey(payload)`
- Returns normalized metrics and `quoteHash`

Purpose:
- Modular validation logic
- Upgradeable pricing engine
- Service-level isolation

---

# 2️⃣ Deployment & Negotiation Flow

## Off-Chain:

1. Carrier and Customer negotiate agreement JSON.
2. Produce canonical `agreementConfigHash`.
3. Customer signs approval message:

`hash(
"APPROVE_DEPLOY",
carrierAddr,
factoryAddr,
customerAccountCodeHash,
agreementConfigHash,
expiry,
nonce
)`

## On-Chain:

4. Carrier calls `onboarding.deployAgreement(...)`
5. Onboarding verifies signature.
6. Onboarding deploys Agreement.
7. Emits `AgreementDeployed` event.

Carrier pays deployment gas (optionally reimbursed via deposit).

---

# 3️⃣ Shipment Creation Flow

1. Customer calls Shipment.
2. Shipment:
   - Validates via Service
   - Requests authorization from Agreement
   - Reserves funds
   - Creates shipment NFT or stored entity
   - Captures funds
3. Emits `ShipmentCreated` event.

---

# 4️⃣ Security Rules

- Agreement only trusts:
  - The Shipment contract specified at deployment
- Shipment only accepts:
  - Agreement deployed by approved onboarding factory
- Onboarding only deploys:
  - Pre-approved code hash
- All signature approvals include:
  - Nonce
  - Expiry
- All billing uses:
  - Reserve → Capture pattern

---

# 5️⃣ Core Design Goals Achieved

- Carrier controls deployment
- Customer cryptographically approves before deployment
- Agreement immutable via config hash
- Per-customer contract isolation
- Modular service-level validation
- On-chain enforcement of pricing and quotas
- Carrier can charge directly via customer contract
- Fully auditable event trail
- Upgrade path via new service contracts

---

# End of Specification