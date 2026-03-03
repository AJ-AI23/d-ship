/**
 * Response types for contract view calls.
 */

/** Tracking event from Tracker.getTrackingStatus. */
export interface TrackingEvent {
  eventType: string;
  timestamp: number;
  location?: string;
}

/** Tracking status response. */
export interface TrackingStatus {
  events: TrackingEvent[];
}

/** Service quote result. */
export interface QuoteResult {
  normalizedMetrics: string;
  amount: bigint;
  quoteHash: string;
  forwarderAddr: string;
  forwarderAmount: bigint;
}

/** Service capabilities. */
export interface ServiceCapabilities {
  defaultPrice: bigint;
  forwarderDefaultFee: bigint;
  carrierAddress: string;
  capabilitiesConfig: string;
}
