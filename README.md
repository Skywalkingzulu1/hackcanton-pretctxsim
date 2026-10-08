# PreTxSim

**Air-gapped local EVM simulation proxy for secure transaction testing**

<p align="center">
  <img src="docs/assets/pretxsim_demo.gif" alt="PreTxSim demo: EIP-7702 delegation flagged, gate refuses; delegatecall+storage tx accepted" width="100%">
</p>

PreTxSim is a lightweight, air-gapped local EVM simulation proxy built in Rust on [`revm`](https://github.com/bluealloy/revm). It intercepts `eth_sendTransaction` calls and runs a full pre-execution structural analysis before any transaction hits the network — ensuring that malicious or buggy transactions are caught before broadcast.

## Key Features

- **Air-gapped**: Simulates transactions locally without exposing private keys or sensitive data to remote RPCs (except for state fetching)
- **Built on `revm`**: High-performance Rust-based EVM implementation with deterministic execution
- **RiskInspector**: Tracks delegatecalls, selfdestructs, storage writes, token transfers, allowance changes, and EIP-7702 delegations
- **Local LLM agent integration**: Optionally consults an offline LLM (Ollama/smollm2) for a PASS/WARN verdict
- **Interactive gatekeeping**: Prompts for confirmation before broadcasting
- **Memory efficient**: Targets <800MB RAM usage

## How It Works

1. Listens on `127.0.0.1:8545` as a local JSON-RPC proxy
2. On `eth_sendTransaction`, intercepts and simulates the transaction against a live RPC (default: `https://eth.llamarpc.com`)
3. Runs the `RiskInspector` to capture pre/post state diffs and flag risky patterns
4. Optionally queries a local LLM agent for a security recommendation
5. Prompts the user to proceed or abort

## Quick Start

### Prerequisites

- Rust (latest stable)
- An upstream RPC URL (EVM-compatible, e.g., Llamarpc, Alchemy, Infura)
- (Optional) Ollama running `smollm2:135m` for AI-powered security briefings

### Build

```bash
cargo build --release
```

### Run

```bash
export RPC_URL="https://eth.llamarpc.com"
./target/release/pretxsim_poc
```

The proxy will start on `127.0.0.1:8545`. Point your wallet, bot, or dApp at this endpoint.

```
======================================================
  PreTxSim Micro-Agent Proxy Live on 127.0.0.1:8545
  Upstream RPC: https://eth.llamarpc.com
  RAM Usage target: < 800 MB
======================================================
```

## Use Cases

- **MEV Bot Safety**: Simulate pending transactions against mempool state before executing
- **Smart Contract Security**: Detect storage modifications, delegatecalls, and selfdestructs pre-flight
- **AI Agent Infrastructure**: Provide a secure simulation layer for autonomous agents before on-chain interaction
- **Foundry Integration**: Complements `forge test` workflows with real-world state simulation

## Project Structure

```
src/
├── main.rs        # Proxy server + JSON-RPC endpoints
├── inspector.rs   # EVM structural analysis (RiskInspector)
├── rpc_db.rs      # RPC-backed database for EVM state lookups
└── agent.rs       # Local LLM agent integration (Ollama)
```

## Configuration

| Env Var   | Default                  | Description                         |
|-----------|--------------------------|-------------------------------------|
| `RPC_URL` | `https://eth.llamarpc.com` | Upstream EVM RPC endpoint        |

## Demo

The recording above shows the real proxy against a deterministic local `anvil` fixture:

- a transaction targeting a **delegated EOA** (EIP-7702 `0xEF0100` designator) is flagged and the gate **refuses** it
- a transaction triggering a real **DELEGATECALL + storage write** is shown with non-zero `RiskInspector` counters and the gate **accepts** it after review
- a local Ollama agent provides the PASS/WARN brief between simulation and gate

Reproduce it end-to-end (requires Rust, Foundry, ffmpeg + [agg](https://github.com/asciinema/agg)):

```bash
cargo build --release
bash demo/render.sh
```

## License

This is a proof-of-concept project. License to be determined.

---

*Built for the W3F Grant Program — enabling secure, air-gapped EVM development for MEV practitioners, security researchers, and AI agent frameworks.*
