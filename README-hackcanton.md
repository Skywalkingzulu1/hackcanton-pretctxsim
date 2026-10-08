# PreTxSim - #HackCanton Season 3 Submission

## Pitch

PreTxSim is a local, air-gapped EVM simulation proxy that sits between your wallet and the network. It intercepts eth_sendTransaction, simulates against live RPC state using evm, and gates broadcast behind interactive confirmation with a local AI security audit.

## Track Alignment: Enterprise Privacy & Contract Logic

#HackCanton's Enterprise Privacy & Logic track requires solutions that respect privacy and verify contract logic before execution. PreTxSim directly addresses this:

- **Air-gapped execution**: All simulation runs locally. Private keys, transaction intents, and state never leave the machine.
- **Zero telemetry**: No third-party services receive transaction data beyond what the user explicitly forwards to their chosen RPC.
- **Local Ollama integration**: The AI advisory (smollm2) runs entirely offline, preserving enterprise-grade privacy.
- **Deterministic verification**: RiskInspector validates contract logic pre-execution (delegatecalls, selfdestructs, storage writes, allowances, EIP-7702).
- **Drop-in privacy-first guardrail**: Single command deployment (docker compose up) with minimal attack surface.

## Technical Stack

- **Rust** + **revm** (high-performance EVM simulation)
- **jsonrpsee** (JSON-RPC proxy server)
- **Ollama** (local LLM for advisory security briefing)
- **Docker Compose** (one-command deployment)

## Demo

- **Live demo (90s)**: [docs/assets/demo.mp4](docs/assets/demo.mp4) (or unlisted YouTube/Loom link)
- **Runbook**: [demo_runbook.md](demo_runbook.md)

## Key Differentiator

Unlike post-hoc scanners, PreTxSim operates at the exact signing moment - intercepting transactions before broadcast. By combining deterministic structural analysis with privacy-preserving local AI, it provides enterprise-ready safety rails without compromising confidentiality.
