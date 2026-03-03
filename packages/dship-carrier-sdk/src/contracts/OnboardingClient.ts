/**
 * Onboarding contract client – carrier deploys Agreement and ForwarderAgreement.
 */

import { Address } from "@multiversx/sdk-core";
import {
  runQuery,
  buildAndSend,
  encodeAddress,
  encodeBuffer,
  encodeBigUint,
  encodeU64,
  toArgHex,
} from "../utils";
import type { Signer } from "../types/config";
import type { DeployAgreementArgs, DeployForwarderAgreementArgs } from "../types/payloads";

export interface OnboardingClientOptions {
  apiUrl: string;
  chainId: string;
  senderAddress: string;
  onboardingAddress: string;
}

export class OnboardingClient {
  constructor(private readonly options: OnboardingClientOptions) {}

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
    return this.options.onboardingAddress;
  }

  /** Deploy customer Agreement. Returns transaction hash. */
  async deployAgreement(signer: Signer, args: DeployAgreementArgs): Promise<string> {
    const enc = {
      customerOwner: encodeAddress(args.customerOwner),
      customerPubkey: toArgHex(args.customerPubkey),
      agreementConfigHash: toArgHex(args.agreementConfigHash),
      shipmentContract: encodeAddress(args.shipmentContract),
      expiry: encodeU64(args.expiry),
      nonce: encodeU64(args.nonce),
      creditLimit: encodeBigUint(args.creditLimit),
      signature: toArgHex(args.signature),
    };
    const callArgs = [
      enc.customerOwner,
      enc.customerPubkey,
      enc.agreementConfigHash,
      enc.shipmentContract,
      enc.expiry,
      enc.nonce,
      enc.creditLimit,
      ...args.enabledServices.map((s) => encodeBuffer(s)),
      enc.signature,
    ];
    const txHash = await buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      this.contract,
      "deployAgreement",
      callArgs,
      signer
    );
    return txHash;
  }

  /** Deploy ForwarderAgreement. Returns tx hash. */
  async deployForwarderAgreement(
    signer: Signer,
    args: DeployForwarderAgreementArgs
  ): Promise<string> {
    const enc = {
      forwarderOwner: encodeAddress(args.forwarderOwner),
      forwarderPubkey: toArgHex(args.forwarderPubkey),
      agreementConfigHash: toArgHex(args.agreementConfigHash),
      shipmentContract: encodeAddress(args.shipmentContract),
      expiry: encodeU64(args.expiry),
      nonce: encodeU64(args.nonce),
      signature: toArgHex(args.signature),
    };
    const callArgs = [
      enc.forwarderOwner,
      enc.forwarderPubkey,
      enc.agreementConfigHash,
      enc.shipmentContract,
      enc.expiry,
      enc.nonce,
      ...args.enabledServices.map((s) => encodeBuffer(s)),
      enc.signature,
    ];
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      this.contract,
      "deployForwarderAgreement",
      callArgs,
      signer
    );
  }

  async setForwarderAgreementTemplate(signer: Signer, template: string): Promise<string> {
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      this.contract,
      "setForwarderAgreementTemplate",
      [encodeAddress(template)],
      signer
    );
  }

  async setAllowedForwarderCodeHash(
    signer: Signer,
    codeHash: string | Uint8Array | Buffer
  ): Promise<string> {
    return buildAndSend(
      this.apiUrl,
      this.chainId,
      this.sender,
      this.contract,
      "setAllowedForwarderCodeHash",
      [toArgHex(codeHash)],
      signer
    );
  }

  async getDeployedAccounts(): Promise<string[]> {
    const result = await runQuery(this.apiUrl, this.contract, "getDeployedAccounts", []);
    return result.map((hex) => Address.fromPubkey(Buffer.from(hex, "hex")).bech32());
  }

  async getAgreementTemplate(): Promise<string> {
    const [hex] = await runQuery(this.apiUrl, this.contract, "getAgreementTemplate", []);
    if (!hex || hex.length === 0) return "";
    return Address.fromPubkey(Buffer.from(hex, "hex")).bech32();
  }

  async getForwarderAgreementTemplate(): Promise<string> {
    const [hex] = await runQuery(
      this.apiUrl,
      this.contract,
      "getForwarderAgreementTemplate",
      []
    );
    if (!hex || hex.length === 0) return "";
    return Address.fromPubkey(Buffer.from(hex, "hex")).bech32();
  }

  async getCarrierAddress(): Promise<string> {
    const [hex] = await runQuery(this.apiUrl, this.contract, "getCarrierAddress", []);
    if (!hex || hex.length === 0) return "";
    return Address.fromPubkey(Buffer.from(hex, "hex")).bech32();
  }
}
