/**
 * Agreement contract client – carrier refund operations.
 * Agreement address is passed per call (carrier manages multiple customer agreements).
 */

import { Address } from "@multiversx/sdk-core";
import {
  runQuery,
  buildAndSend,
  encodeAddress,
  encodeBuffer,
  encodeBigUint,
} from "../utils";
import type { Signer } from "../types/config";

export interface AgreementClientOptions {
  apiUrl: string;
  chainId: string;
  senderAddress: string;
}

export class AgreementClient {
  constructor(private readonly options: AgreementClientOptions) {}

  private get apiUrl() {
    return this.options.apiUrl;
  }
  private get chainId() {
    return this.options.chainId;
  }
  private get sender() {
    return this.options.senderAddress;
  }

  /** Refund voided shipment. Carrier sends EGLD = captured amount. */
  async refundForVoidedShipment(
    signer: Signer,
    agreementAddr: string,
    trackingNumber: string,
    amount: bigint
  ): Promise<string> {
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      agreementAddr,
      "refundForVoidedShipment",
      [encodeBuffer(trackingNumber)],
      signer,
      amount
    );
  }

  /** Refund for return shipment (exception case). Carrier sends EGLD = captured amount. */
  async refundForReturnShipment(
    signer: Signer,
    agreementAddr: string,
    outboundTrackingNumber: string,
    amount: bigint
  ): Promise<string> {
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      agreementAddr,
      "refundForReturnShipment",
      [encodeBuffer(outboundTrackingNumber)],
      signer,
      amount
    );
  }

  async getCustomerOwner(agreementAddr: string): Promise<string> {
    const [hex] = await runQuery(
      this.apiUrl,
      agreementAddr,
      "getCustomerOwner",
      []
    );
    if (!hex || hex.length === 0) return "";
    return Address.fromPubkey(Buffer.from(hex, "hex")).bech32();
  }

  async getDepositBalance(agreementAddr: string): Promise<bigint> {
    const [hex] = await runQuery(
      this.apiUrl,
      agreementAddr,
      "getDepositBalance",
      []
    );
    return hex ? BigInt("0x" + hex) : 0n;
  }

  async getReservation(
    agreementAddr: string,
    reservationId: number
  ): Promise<{ amount: bigint; reference: string; state: number } | null> {
    const encId = Buffer.alloc(8);
    encId.writeBigUInt64BE(BigInt(reservationId));
    const result = await runQuery(
      this.apiUrl,
      agreementAddr,
      "getReservation",
      [encId.toString("hex")]
    );
    if (!result || result.length < 3) return null;
    return {
      amount: result[0] ? BigInt("0x" + result[0]) : 0n,
      reference: result[1] ? Buffer.from(result[1], "hex").toString("utf8") : "",
      state: result[2] ? parseInt(result[2], 16) : 0,
    };
  }
}
