/**
 * Base contract interaction helper.
 * Uses MultiversX API for VM queries and builds transactions for execution.
 */

import { Transaction, Address } from "@multiversx/sdk-core";
import {
  encodeAddress,
  encodeBuffer,
  encodeBigUint,
} from "./utils/encoding";
import type { Signer } from "./types/config";
import { getApiUrl, getChainId } from "./config";

const VM_QUERY_PATH = "/vm-values/query";
const TRANSACTIONS_PATH = "/transaction/send";

export interface ContractBaseOptions {
  apiUrl: string;
  chainId: string;
}

/**
 * Run a view/query on a smart contract.
 */
export async function runQuery(
  apiUrl: string,
  scAddress: string,
  funcName: string,
  args: string[]
): Promise<string[]> {
  const payload = {
    scAddress: Address.fromBech32(scAddress).hex(),
    funcName,
    args: args.map((hex) => Buffer.from(hex, "hex").toString("base64")),
  };

  const res = await fetch(`${apiUrl}${VM_QUERY_PATH}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`VM query failed: ${res.status} ${text}`);
  }

  const data = await res.json();
  const values = data.data?.data?.returnData ?? [];
  return values.map((b64: string) =>
    Buffer.from(b64, "base64").toString("hex")
  );
}

/**
 * Build transaction data for contract call: functionName@arg1@arg2...
 */
export function buildCallData(
  funcName: string,
  args: string[]
): string {
  const parts = [funcName, ...args.filter((a) => a !== undefined && a !== null)];
  return parts.join("@");
}

/** Get account nonce for transaction building */
export async function getAccountNonce(
  apiUrl: string,
  address: string
): Promise<number> {
  const res = await fetch(`${apiUrl}/address/${address}`);
  if (!res.ok) return 0;
  const data = await res.json();
  return data.data?.account?.nonce ?? data.nonce ?? 0;
}

/**
 * Create and sign a transaction, then broadcast it.
 * The signer receives a Transaction and must return the signed transaction.
 * Nonce is fetched from API if tx has nonce 0.
 */
export async function buildAndSend(
  apiUrl: string,
  chainId: string,
  sender: string,
  contractAddress: string,
  funcName: string,
  args: string[],
  signer: Signer,
  value: bigint = 0n,
  nonce?: number
): Promise<string> {
  const data = buildCallData(funcName, args);
  const resolvedNonce =
    nonce ?? (await getAccountNonce(apiUrl, sender));

  const tx = new Transaction({
    nonce: BigInt(resolvedNonce),
    value,
    sender: Address.fromBech32(sender),
    receiver: Address.fromBech32(contractAddress),
    gasLimit: 60000000n,
    gasPrice: 1000000000n,
    chainID: chainId,
    data: new Uint8Array(Buffer.from(data, "utf8")),
  });

  const signed = await signer(tx);

  const broadcastPayload: Record<string, unknown> = {
    nonce: Number(signed.nonce),
    value: signed.value.toString(),
    receiver: contractAddress,
    sender,
    gasPrice: 1000000000,
    gasLimit: signed.gasLimit?.toString() ?? "60000000",
    data: Buffer.from(data, "utf8").toString("base64"),
    chainID: chainId,
    version: signed.version ?? 1,
  };

  if (signed.signature && signed.signature.length > 0) {
    broadcastPayload.signature = Buffer.from(signed.signature).toString("hex");
  }

  const res = await fetch(`${apiUrl}${TRANSACTIONS_PATH}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(broadcastPayload),
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`Transaction failed: ${res.status} ${text}`);
  }

  const result = await res.json();
  return result.txHash ?? result.data?.txHash ?? result.hash ?? "";
}

/** Helper to ensure hex has even length */
function ensureHex(s: string): string {
  if (!s) return "";
  return s.length % 2 === 0 ? s : "0" + s;
}

export { encodeAddress, encodeBuffer, encodeBigUint, ensureHex };
