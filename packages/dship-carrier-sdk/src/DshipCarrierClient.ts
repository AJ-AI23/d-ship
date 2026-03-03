/**
 * DshipCarrierClient – central entry point for carrier backend integrations.
 */

import type { DshipCarrierConfig } from "./types/config";
import { getApiUrl, getChainId } from "./config";
import { OnboardingClient } from "./contracts/OnboardingClient";
import { ServiceClient } from "./contracts/ServiceClient";
import { ShipmentClient } from "./contracts/ShipmentClient";
import { TrackerClient } from "./contracts/TrackerClient";
import { AgreementClient } from "./contracts/AgreementClient";

export class DshipCarrierClient {
  readonly onboarding: OnboardingClient;
  readonly service: ServiceClient;
  readonly shipment: ShipmentClient;
  readonly tracker: TrackerClient;
  readonly agreement: AgreementClient;

  constructor(config: DshipCarrierConfig) {
    const apiUrl = getApiUrl(config.network, config.apiUrl);
    const chainId = getChainId(config.network);
    const { addresses, senderAddress } = config;

    const base = { apiUrl, chainId, senderAddress };

    this.onboarding = new OnboardingClient({
      ...base,
      onboardingAddress: addresses.onboarding,
    });

    this.service = new ServiceClient({
      ...base,
      serviceAddress: addresses.service,
    });

    this.shipment = new ShipmentClient({
      ...base,
      shipmentAddress: addresses.shipment,
    });

    this.tracker = new TrackerClient({
      ...base,
      trackerAddress: addresses.tracker,
    });

    this.agreement = new AgreementClient(base);
  }
}
