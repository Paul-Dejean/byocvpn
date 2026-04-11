# On-Demand VPN

A multi-cloud desktop app that spins up ephemeral WireGuard VPN servers on your own cloud accounts. No subscription, no third-party servers — you own the infrastructure.

Supports **AWS**, **Azure**, **GCP**, and **Oracle Cloud**.

## How it works

1. Connect your cloud account credentials
2. Pick a region and deploy a VPN server in seconds
3. Connect to it via WireGuard
4. Terminate it when you're done — pay only for what you use

## Architecture

Built with Rust (Tauri 2) backend and React/TypeScript frontend.

| Crate            | Description                                             |
| ---------------- | ------------------------------------------------------- |
| `byocvpn_core`   | Shared types, WireGuard tunnel management, IPC protocol |
| `byocvpn_aws`    | AWS provider (EC2)                                      |
| `byocvpn_azure`  | Azure provider                                          |
| `byocvpn_gcp`    | GCP provider                                            |
| `byocvpn_oracle` | Oracle Cloud provider                                   |
| `byocvpn_daemon` | Background service managing WireGuard tunnels           |
| `byocvpn_cli`    | CLI frontend                                            |
| `ui`             | Tauri + React frontend                                  |

## License

Copyright (C) 2025 Paul Dejean. Licensed under the [GNU General Public License v3.0](LICENSE).
