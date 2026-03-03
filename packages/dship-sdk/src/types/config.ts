/**
 * Configuration types for the d-ship SDK.
 */

/** Carrier contract addresses (bech32). */
export interface CarrierAddresses {
  shipment: string;
  agreement?: string;
  parcel: string;
  tracker: string;
  pickup: string;
  onboarding: string;
}

/** SDK configuration. */
export interface DshipConfig {
  /** Network identifier. */
  network: "devnet" | "mainnet" | "testnet";
  /** Override API/gateway URL. Uses default per network if not set. */
  apiUrl?: string;
  /** Carrier contract addresses. */
  addresses: CarrierAddresses;
}

/** Signer: takes a transaction, signs it, returns signed transaction. */
export type Signer = (tx: import("@multiversx/sdk-core").Transaction) => Promise<import("@multiversx/sdk-core").Transaction>;
