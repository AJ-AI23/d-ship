/**
 * d-ship SDK – TypeScript client for customer and consumer integrations.
 *
 * Usage:
 * ```ts
 * const client = new DshipClient(config);
 * client.agreement.setAddress(agreementAddr);
 * const status = await client.tracker.getTrackingStatus(trackingNumber);
 * await client.shipment.createShipment(signer, { agreementAddr, serviceId, payload, parcelIds });
 * ```
 */

export { DshipClient } from "./DshipClient";
export { DshipConfig, CarrierAddresses } from "./config";
export type { Signer } from "./types/config";
export { AgreementClient } from "./contracts/AgreementClient";
export { ShipmentClient } from "./contracts/ShipmentClient";
export { ParcelClient } from "./contracts/ParcelClient";
export { TrackerClient } from "./contracts/TrackerClient";
export { PickupClient } from "./contracts/PickupClient";
export { ServiceClient } from "./contracts/ServiceClient";
export * from "./types/payloads";
export * from "./types/responses";
export { encodeAddress, decodeAddress, toHex, fromHex } from "./utils/encoding";
export {
  deriveReceiverKey,
  decryptWithReceiverSecret,
  encryptForReceiver,
} from "./utils/kdf";
