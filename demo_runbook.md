# PreTxSim Demo Runbook

This guide is for hackathon evaluators to reproduce the core demo in < 3 minutes.

## Prerequisites

- [Docker](https://www.docker.com/) and Docker Compose
- [Foundry](https://book.getfoundry.sh/) (optional) - or any local RPC for full simulation
- [Ollama](https://ollama.ai/) if running locally outside Docker (optional)

## Quick Start (Single Command)

1. **Start PreTxSim + Ollama**
`ash
docker compose up -d
`

2. **Configure Environment** (if needed)
`ash
export RPC_URL=https://eth.llamarpc.com  # or your preferred RPC
`

3. **Run Demo Transations**

Point your wallet, curl, or testing script to http://127.0.0.1:8545.

Example (using curl):
`ash
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
`

## Demo Payloads

The examples/demo_txs.json file contains two sample transactions:

- **Payload 1 (Malicious/Blocked)**: Flags delegation/allowance risk patterns at the gate
- **Payload 2 (Clean/Passed)**: Standard transfer showing revm state trace, balance deltas, gas estimate, and Ollama PASS summary

## Architecture

- **Air-gapped simulation**: Runs locally via evm with no keys broadcast until accepted
- **RiskInspector**: Detects delegatecalls, selfdestructs, storage writes, allowances, EIP-7702 delegations
- **Local LLM Auditor**: Ollama smollm2 provides advisory PASS/WARN brief
- **Proxy Gate**: Interactive approval/refusal before network broadcast

## CI Mode

For automated testing:
`ash
cat examples/demo_txs.json | ./target/release/pretxsim_poc --ci
`

Exits with code 1 if any BLOCK rules are triggered.
