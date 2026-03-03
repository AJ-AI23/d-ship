/**
 * Carrier SDK configuration and network helpers.
 */

import type { CarrierAddresses } from "./types/config";

export type { DshipCarrierConfig, CarrierAddresses } from "./types/config";

/** Default API URLs per network */
export const DEFAULT_API_URLS: Record<string, string> = {
  devnet: "https://devnet-api.multiversx.com",
  mainnet: "https://api.multiversx.com",
  testnet: "https://testnet-api.multiversx.com",
};

/** Chain IDs per network */
export const CHAIN_IDS: Record<string, string> = {
  devnet: "D",
  mainnet: "1",
  testnet: "T",
};

/**
 * Resolves API URL for the given network.
 */
export function getApiUrl(network: string, override?: string): string {
  return override ?? DEFAULT_API_URLS[network] ?? DEFAULT_API_URLS.devnet;
}

/**
 * Resolves chain ID for the given network.
 */
export function getChainId(network: string): string {
  return CHAIN_IDS[network] ?? CHAIN_IDS.devnet;
}
