/**
 * Tracker contract client – carrier registers events and configures contracts.
 */

import { Address } from "@multiversx/sdk-core";
import { runQuery, buildAndSend, encodeAddress, encodeBuffer, encodeU64 } from "../utils";
import type { Signer } from "../types/config";
import type { TrackingStatus } from "../types/responses";

export interface TrackerClientOptions {
  apiUrl: string;
  chainId: string;
  senderAddress: string;
  trackerAddress: string;
}

export class TrackerClient {
  constructor(private readonly options: TrackerClientOptions) {}

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
    return this.options.trackerAddress;
  }

  async registerEvent(
    signer: Signer,
    trackingNumber: string,
    eventType: string,
    timestamp: number,
    location?: string | Uint8Array | Buffer
  ): Promise<string> {
    const args = [
      encodeBuffer(trackingNumber),
      encodeBuffer(eventType),
      encodeU64(timestamp),
      ...(location !== undefined ? [encodeBuffer(location)] : []),
    ];
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      this.contract,
      "registerEvent",
      args,
      signer
    );
  }

  async setShipmentContract(signer: Signer, addr: string): Promise<string> {
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      this.contract,
      "setShipmentContract",
      [encodeAddress(addr)],
      signer
    );
  }

  async setPickupContract(signer: Signer, addr: string): Promise<string> {
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      this.contract,
      "setPickupContract",
      [encodeAddress(addr)],
      signer
    );
  }

  async getTrackingStatus(trackingNumber: string): Promise<TrackingStatus> {
    const result = await runQuery(this.apiUrl, this.contract, "getTrackingStatus", [
      encodeBuffer(trackingNumber),
    ]);
    const events: { eventType: string; timestamp: number; location?: string }[] = [];
    for (let i = 0; i < result.length; i += 3) {
      if (i + 2 < result.length) {
        const eventType = Buffer.from(result[i], "hex").toString("utf8");
        const timestamp = parseInt(result[i + 1], 16) || 0;
        const locHex = result[i + 2];
        const location = locHex ? Buffer.from(locHex, "hex").toString("utf8") : undefined;
        events.push({ eventType, timestamp, location });
      }
    }
    return { events };
  }

  async hasDispatched(trackingNumber: string): Promise<boolean> {
    const [hex] = await runQuery(this.apiUrl, this.contract, "hasDispatched", [
      encodeBuffer(trackingNumber),
    ]);
    return hex === "01" || hex === "74727565";
  }

  async hasEventType(trackingNumber: string, eventType: string): Promise<boolean> {
    const [hex] = await runQuery(this.apiUrl, this.contract, "hasEventType", [
      encodeBuffer(trackingNumber),
      encodeBuffer(eventType),
    ]);
    return hex === "01" || hex === "74727565";
  }

  async getAgreementForShipment(trackingNumber: string): Promise<string | null> {
    const [hex] = await runQuery(this.apiUrl, this.contract, "getAgreementForShipment", [
      encodeBuffer(trackingNumber),
    ]);
    if (!hex || hex.length < 64) return null;
    return Address.fromPubkey(Buffer.from(hex, "hex")).bech32();
  }

  async hasReimbursableReturn(outboundTrackingNumber: string): Promise<boolean> {
    const [hex] = await runQuery(this.apiUrl, this.contract, "hasReimbursableReturn", [
      encodeBuffer(outboundTrackingNumber),
    ]);
    return hex === "01" || hex === "74727565";
  }
}
