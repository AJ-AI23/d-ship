/**
 * Service contract client – carrier manages forwarder registry; shared validate/quote views.
 */

import { Address } from "@multiversx/sdk-core";
import {
  runQuery,
  buildAndSend,
  encodeAddress,
  encodeBuffer,
  encodeBigUint,
  toArgHex,
} from "../utils";
import type { Signer } from "../types/config";
import type { ServiceCapabilities, QuoteResult } from "../types/responses";

export interface ServiceClientOptions {
  apiUrl: string;
  chainId: string;
  senderAddress: string;
  serviceAddress: string;
}

export class ServiceClient {
  constructor(private readonly options: ServiceClientOptions) {}

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
    return this.options.serviceAddress;
  }

  async setCarrierAddress(signer: Signer, carrier: string): Promise<string> {
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      this.contract,
      "setCarrierAddress",
      [encodeAddress(carrier)],
      signer
    );
  }

  async registerForwarder(signer: Signer, forwarderAgreementAddr: string): Promise<string> {
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      this.contract,
      "registerForwarder",
      [encodeAddress(forwarderAgreementAddr)],
      signer
    );
  }

  async unregisterForwarder(signer: Signer, addr: string): Promise<string> {
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      this.contract,
      "unregisterForwarder",
      [encodeAddress(addr)],
      signer
    );
  }

  async validate(
    payload: string | Uint8Array | Buffer,
    forwarderAgreementAddr?: string
  ): Promise<boolean> {
    const args =
      forwarderAgreementAddr !== undefined
        ? [toArgHex(payload), encodeAddress(forwarderAgreementAddr)]
        : [toArgHex(payload)];
    const result = await runQuery(this.apiUrl, this.contract, "validate", args);
    if (!result || result.length === 0) return false;
    const val = result[0];
    return val === "01" || val === "74727565"; // true
  }

  async quote(
    payload: string | Uint8Array | Buffer,
    forwarderAgreementAddr?: string
  ): Promise<QuoteResult> {
    const args =
      forwarderAgreementAddr !== undefined
        ? [toArgHex(payload), encodeAddress(forwarderAgreementAddr)]
        : [toArgHex(payload)];
    const result = await runQuery(this.apiUrl, this.contract, "quote", args);
    if (!result || result.length < 5) {
      throw new Error("Invalid quote response");
    }
    const [normHex, amountHex, hashHex, fwdHex, fwdAmountHex] = result;
    return {
      normalizedMetrics: Buffer.from(normHex, "hex").toString("utf8"),
      amount: amountHex ? BigInt("0x" + amountHex) : 0n,
      quoteHash: hashHex,
      forwarderAddr: fwdHex
        ? Address.fromPubkey(Buffer.from(fwdHex, "hex")).bech32()
        : "",
      forwarderAmount: fwdAmountHex ? BigInt("0x" + fwdAmountHex) : 0n,
    };
  }

  async getCapabilities(): Promise<ServiceCapabilities> {
    const result = await runQuery(this.apiUrl, this.contract, "getCapabilities", []);
    if (!result || result.length < 4) {
      throw new Error("Invalid getCapabilities response");
    }
    const [priceHex, feeHex, carrierHex, configHex] = result;
    return {
      defaultPrice: priceHex ? BigInt("0x" + priceHex) : 0n,
      forwarderDefaultFee: feeHex ? BigInt("0x" + feeHex) : 0n,
      carrierAddress: carrierHex
        ? Address.fromPubkey(Buffer.from(carrierHex, "hex")).bech32()
        : "",
      capabilitiesConfig: configHex ? Buffer.from(configHex, "hex").toString("utf8") : "",
    };
  }

  async getRegisteredForwarderCount(): Promise<number> {
    const [hex] = await runQuery(
      this.apiUrl,
      this.contract,
      "getRegisteredForwarderCount",
      []
    );
    return hex ? parseInt(hex, 16) : 0;
  }

  async getRegisteredForwarders(offset: number, limit: number): Promise<string[]> {
    const encOffset = Buffer.alloc(4);
    encOffset.writeUInt32BE(offset);
    const encLimit = Buffer.alloc(4);
    encLimit.writeUInt32BE(Math.min(limit, 100));
    const result = await runQuery(this.apiUrl, this.contract, "getRegisteredForwarders", [
      encOffset.toString("hex"),
      encLimit.toString("hex"),
    ]);
    return result.map((hex) => Address.fromPubkey(Buffer.from(hex, "hex")).bech32());
  }
}
