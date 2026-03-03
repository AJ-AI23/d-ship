/**
 * Payload types aligned with d-ship schemas.
 * Reference: schemas/shipment-booking.schema.json, schemas/parcel.schema.json
 */

/** Shipment booking payload (JSON). Forwarded to Service.validate/quote. */
export interface ShipmentBooking {
  shipmentChannelId: string;
  parties: Party[];
  carrierDefinitionId: string;
  serviceDefinitionId: string;
  parcelIds?: string[];
  forwarderAgreementAddr?: string;
  [key: string]: unknown;
}

/** Party in shipment (sender, receiver, etc.). */
export interface Party {
  partyId: string;
  role?: string;
  type?: string;
  forwarderAgreementAddr?: string;
  [key: string]: unknown;
}

/** Parcel registration arguments. */
export interface RegisterParcelArgs {
  parcelId: string;
  reference?: string;
  description?: string;
  weight: number;
  weightUnit: "G" | "KG" | "LB" | "OZ";
  itemIds?: string[];
  serial?: string;
  dgNetQuantity?: number;
  dgNetQuantityUnit?: string;
  dgTypeCode?: string;
  dgQuantity?: number;
}

/** Create shipment arguments. */
export interface CreateShipmentArgs {
  agreementAddr: string;
  serviceId: string;
  payload: string | Uint8Array | Buffer;
  parcelIds: string[];
  forwarderAddr?: string;
  encryptedPayload?: string | Uint8Array | Buffer;
}

/** Request pickup arguments. */
export interface RequestPickupArgs {
  trackingNumbers: string[];
  agreementAddr: string;
  slotDate: number;
  slotTimeFrom: string;
  slotTimeTo: string;
  location?: string | Uint8Array | Buffer;
}
