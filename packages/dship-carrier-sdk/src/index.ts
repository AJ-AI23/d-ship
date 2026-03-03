/**
 * d-ship Carrier SDK – TypeScript client for carrier backend integrations.
 *
 * Usage:
 * ```ts
 * const client = new DshipCarrierClient(config);
 * await client.onboarding.deployAgreement(signer, args);
 * await client.service.registerForwarder(signer, forwarderAddr);
 * await client.shipment.createReturnShipment(signer, args);
 * await client.tracker.registerEvent(signer, tn, "IN_TRANSIT", Date.now()/1000);
 * await client.agreement.refundForVoidedShipment(signer, agreementAddr, tn, amount);
 * ```
 */

export { DshipCarrierClient } from "./DshipCarrierClient";
export type { DshipCarrierConfig, CarrierAddresses } from "./config";
export type { Signer } from "./types/config";
export { OnboardingClient } from "./contracts/OnboardingClient";
export { ServiceClient } from "./contracts/ServiceClient";
export { ShipmentClient } from "./contracts/ShipmentClient";
export { TrackerClient } from "./contracts/TrackerClient";
export { AgreementClient } from "./contracts/AgreementClient";
export * from "./types/payloads";
export * from "./types/responses";
export { encodeAddress, decodeAddress, toHex, fromHex, encodeU64, toArgHex } from "./utils";
