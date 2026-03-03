/**
 * Shipment contract client – carrier admin and createReturnShipment.
 */

import { Address } from "@multiversx/sdk-core";
import {
  runQuery,
  buildAndSend,
  encodeAddress,
  encodeBuffer,
  toArgHex,
} from "../utils";
import type { Signer } from "../types/config";
import type { CreateReturnShipmentArgs } from "../types/payloads";

export interface ShipmentClientOptions {
  apiUrl: string;
  chainId: string;
  senderAddress: string;
  shipmentAddress: string;
}

export class ShipmentClient {
  constructor(private readonly options: ShipmentClientOptions) {}

  private get apiUrl() {
    return this.options.apiUrl;
  }
  private get chainId() {
    return this.options.chainId;
  }
  private get sender() {
    return this.options.senderAddress;
  }
  private get contract() {
    return this.options.shipmentAddress;
  }

  async registerService(
    signer: Signer,
    serviceId: string,
    serviceAddr: string
  ): Promise<string> {
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      this.contract,
      "registerService",
      [encodeBuffer(serviceId), encodeAddress(serviceAddr)],
      signer
    );
  }

  async setParcelContract(signer: Signer, parcelContract: string): Promise<string> {
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      this.contract,
      "setParcelContract",
      [encodeAddress(parcelContract)],
      signer
    );
  }

  async setPickupContract(signer: Signer, pickupContract: string): Promise<string> {
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      this.contract,
      "setPickupContract",
      [encodeAddress(pickupContract)],
      signer
    );
  }

  async createReturnShipment(
    signer: Signer,
    args: CreateReturnShipmentArgs
  ): Promise<string> {
    const callArgs = [
      encodeBuffer(args.outboundTrackingNumber),
      encodeAddress(args.agreementAddr),
      encodeBuffer(args.serviceId),
      toArgHex(args.payload),
      ...args.parcelIds.map((p) => encodeBuffer(p)),
      args.reimburseShipper ? "01" : "00",
      ...(args.encryptedPayload !== undefined
        ? [toArgHex(args.encryptedPayload)]
        : []),
    ];
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      this.contract,
      "createReturnShipment",
      callArgs,
      signer
    );
  }

  async hasShipment(trackingNumber: string): Promise<boolean> {
    const [hex] = await runQuery(this.apiUrl, this.contract, "hasShipment", [
      encodeBuffer(trackingNumber),
    ]);
    return hex === "01" || hex === "74727565";
  }

  async isVoided(trackingNumber: string): Promise<boolean> {
    const [hex] = await runQuery(this.apiUrl, this.contract, "isVoided", [
      encodeBuffer(trackingNumber),
    ]);
    return hex === "01" || hex === "74727565";
  }

  async getOutboundForReturn(trackingNumber: string): Promise<string | null> {
    const result = await runQuery(this.apiUrl, this.contract, "getOutboundForReturn", [
      encodeBuffer(trackingNumber),
    ]);
    if (!result || result.length === 0 || !result[0]) return null;
    const buf = Buffer.from(result[0], "hex");
    if (buf.length === 0) return null;
    return buf.toString("utf8");
  }

  async getServiceAddress(serviceId: string): Promise<string | null> {
    const result = await runQuery(this.apiUrl, this.contract, "getServiceAddress", [
      encodeBuffer(serviceId),
    ]);
    if (!result || result.length === 0 || !result[0]) return null;
    const hex = result[0];
    if (hex.length < 64) return null;
    return Address.fromPubkey(Buffer.from(hex, "hex")).bech32();
  }
}
