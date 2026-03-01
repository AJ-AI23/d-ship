# MultiverseX Shipping d-apps

Master repository defining the foundation for **decentralized shipping d-apps** on MultiverseX. Carrier forks inherit this structure and deploy their services as uniquely configured d-apps via the platform webapp.

## Structure

```
mvx-dapps/
├── .github/workflows/
│   └── deploy.yml      # Platform-triggered deployment (network, contract_path, dapp_id, config)
├── contracts/
│   ├── shipment-creation/   # Shipment creation template (githubPath)
│   │   ├── src/
│   │   ├── meta/
│   │   ├── wasm/
│   │   └── Cargo.toml
│   ├── tracking-event/     # Register tracking events
│   ├── serial-number/      # Serial number generation
│   └── ...                 # More d-app templates
├── config/
│   └── shipment-creation.example.json
└── README.md
```

## D-app Categories

| Contract           | Purpose                        |
|--------------------|--------------------------------|
| `shipment-creation`| Create shipments with validation|
| `tracking-event`   | Register tracking events       |
| `serial-number`    | Generate serial numbers        |

Each d-app accepts a **JSON config** at init/upgrade, enabling carrier-specific rules without code changes.

## Fork & Deploy

1. **Fork** this repository to your organization (e.g. `mycarrier/mvx-dapps`).
2. **Connect** the fork to the platform webapp (GitHub OAuth).
3. **Deploy** a d-app by choosing a template, providing config, and triggering the workflow.

The platform triggers `deploy.yml` with:

- `network` – mainnet, devnet, testnet
- `contract_path` – e.g. `contracts/shipment-creation`
- `dapp_id` – unique deployment ID
- `config` – JSON configuration

## Example Config (shipment-creation)

```json
{
  "network": {
    "gasLimit": 60000000
  },
  "processors": {
    "autoLabel": true,
    "barcodeFormat": "CODE128"
  },
  "validators": {
    "maxWeight": 30000,
    "maxDimensions": {
      "width": 200,
      "height": 200,
      "length": 300
    }
  }
}
```

Carrier forks can extend this schema for their own use. The config is passed to the contract at init/upgrade and stored on-chain.

## Building Locally

```bash
# Install sc-meta
cargo install multiversx-sc-meta --version 0.59.0

# Build a single contract
cd contracts/shipment-creation
sc-meta all build --locked

# Or use the reproducible Docker build
wget https://raw.githubusercontent.com/multiversx/mx-sdk-rust-contract-builder/v10.0.0/build_with_docker.py
python3 build_with_docker.py --image=multiversx/sdk-rust-contract-builder:v10.0.0 --project=. --contract=shipment-creation --output=./output
```

## Admin vs Carrier

- **Admin users** (in the webapp): Upload/fork code, manage templates, overview all d-apps.
- **Carrier users**: Deploy their d-apps from templates with config, view their own deployments.

---

*Part of the MultiverseX shipping ecosystem.*
