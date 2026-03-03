/**
 * Encoding utilities for MultiversX contract arguments.
 */

import { Address } from "@multiversx/sdk-core";

export function encodeAddress(bech32: string): string {
  const addr = Address.fromBech32(bech32);
  return Buffer.from(addr.pubkey()).toString("hex");
}

export function decodeAddress(hex: string, prefix = "erd"): string {
  const buf = Buffer.from(hex, "hex");
  return Address.fromPubkey(buf).bech32(prefix);
}

export function toHex(data: Uint8Array | Buffer | string): string {
  if (typeof data === "string") {
    return Buffer.from(data, "utf8").toString("hex");
  }
  return Buffer.from(data).toString("hex");
}

export function fromHex(hex: string): Buffer {
  return Buffer.from(hex, "hex");
}

export function encodeBigUint(value: bigint): string {
  if (value === 0n) return "";
  const hex = value.toString(16);
  return hex.length % 2 === 0 ? hex : "0" + hex;
}

export function encodeBuffer(value: string | Uint8Array | Buffer): string {
  if (typeof value === "string") {
    return Buffer.from(value, "utf8").toString("hex");
  }
  return Buffer.from(value).toString("hex");
}

/** Encode u64 as 8-byte big-endian hex. */
export function encodeU64(value: number | bigint): string {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64BE(BigInt(value));
  return buf.toString("hex");
}

/** Normalize bytes-like value to hex. Handles hex string, base64, Buffer, Uint8Array. */
export function toArgHex(value: string | Uint8Array | Buffer): string {
  if (typeof value === "string") {
    const t = value.trim();
    if (t.length % 2 === 0 && /^[0-9a-fA-F]+$/.test(t)) return t;
    try {
      return Buffer.from(t, "base64").toString("hex");
    } catch {
      return Buffer.from(t, "utf8").toString("hex");
    }
  }
  return Buffer.from(value).toString("hex");
}
