/**
 * Payload types for carrier operations.
 * Reference: docs/contract-payloads.md
 */

/** Deploy customer Agreement arguments. */
export interface DeployAgreementArgs {
  customerOwner: string;
  customerPubkey: string | Uint8Array | Buffer;
  agreementConfigHash: string | Uint8Array | Buffer;
  shipmentContract: string;
  expiry: number;
  nonce: number;
  creditLimit: bigint;
  enabledServices: string[];
  signature: string | Uint8Array | Buffer;
}

/** Deploy ForwarderAgreement arguments. */
export interface DeployForwarderAgreementArgs {
  forwarderOwner: string;
  forwarderPubkey: string | Uint8Array | Buffer;
  agreementConfigHash: string | Uint8Array | Buffer;
  shipmentContract: string;
  expiry: number;
  nonce: number;
  enabledServices: string[];
  signature: string | Uint8Array | Buffer;
}

/** Create return shipment arguments (carrier only). */
export interface CreateReturnShipmentArgs {
  outboundTrackingNumber: string;
  agreementAddr: string;
  serviceId: string;
  payload: string | Uint8Array | Buffer;
  parcelIds: string[];
  reimburseShipper: boolean;
  encryptedPayload?: string | Uint8Array | Buffer;
}

/** Register tracking event arguments. */
export interface RegisterEventArgs {
  trackingNumber: string;
  eventType: string;
  timestamp: number;
  location?: string | Uint8Array | Buffer;
}
