/**
 * Encoding utilities for MultiversX contract arguments.
 * Arguments are passed as hex strings (even length).
 */

import { Address } from "@multiversx/sdk-core";

/**
 * Encode an address (bech32) to hex for contract arguments.
 */
export function encodeAddress(bech32: string): string {
  const addr = Address.fromBech32(bech32);
  return Buffer.from(addr.pubkey()).toString("hex");
}

/**
 * Decode hex to bech32 address. Requires chain prefix (erd for mainnet).
 */
export function decodeAddress(hex: string, prefix = "erd"): string {
  const buf = Buffer.from(hex, "hex");
  return Address.fromPubkey(buf).bech32(prefix);
}

/**
 * Encode bytes/Buffer to hex.
 */
export function toHex(data: Uint8Array | Buffer | string): string {
  if (typeof data === "string") {
    return Buffer.from(data, "utf8").toString("hex");
  }
  return Buffer.from(data).toString("hex");
}

/**
 * Decode hex to Buffer.
 */
export function fromHex(hex: string): Buffer {
  return Buffer.from(hex, "hex");
}

/**
 * Encode a BigInt amount to hex (for BigUint contract arg).
 * MultiversX expects compact hex; zero is empty string.
 */
export function encodeBigUint(value: bigint): string {
  if (value === 0n) return "";
  const hex = value.toString(16);
  return hex.length % 2 === 0 ? hex : "0" + hex;
}

/**
 * Encode ManagedBuffer (utf8 string or bytes) to hex.
 */
export function encodeBuffer(value: string | Uint8Array | Buffer): string {
  if (typeof value === "string") {
    return Buffer.from(value, "utf8").toString("hex");
  }
  return Buffer.from(value).toString("hex");
}
