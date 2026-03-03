/**
 * Configuration types for the d-ship Carrier SDK.
 */

/** Carrier contract addresses (bech32). */
export interface CarrierAddresses {
  onboarding: string;
  shipment: string;
  tracker: string;
  pickup: string;
  service: string;
}

/** Carrier SDK configuration. */
export interface DshipCarrierConfig {
  /** Network identifier. */
  network: "devnet" | "mainnet" | "testnet";
  /** Override API/gateway URL. Uses default per network if not set. */
  apiUrl?: string;
  /** Carrier contract addresses. */
  addresses: CarrierAddresses;
  /** Carrier wallet address (sender for transactions). Required for write operations. */
  senderAddress: string;
}

/** Signer: takes a transaction, signs it, returns signed transaction. */
export type Signer = (
  tx: import("@multiversx/sdk-core").Transaction
) => Promise<import("@multiversx/sdk-core").Transaction>;
