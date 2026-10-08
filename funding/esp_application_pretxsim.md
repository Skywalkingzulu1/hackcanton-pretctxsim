# PreTxSim — Ethereum ESP Direct Grant Application

## Contact Information

- **First Name:** PreTxSim
- **Last Name:** Maintainers
- **Email:** pretxsim@proton.me
- **Company:** PreTxSim Core
- **Alternative Contact Info:** N/A
- **Website:** https://github.com/Skywalkingzulu1/pretxsim_poc
- **City:** N/A
- **Country:** International
- **Time Zone:** UTC

## Applicant Profile

PreTxSim is an open-source, air-gapped local EVM simulation proxy built in Rust on revm. The lead developer is an independent blockchain security researcher with deep experience in EVM internals, Rust systems programming, and open-source tooling. The project is maintained as a community-driven initiative with contributions from the Web3 security and MEV botting communities.

## Budget

- **Budget Request:** $20,000 USD
- **Currency:** USD

## Project Overview

- **Project Name:** PreTxSim — Air-Gapped EVM Simulation Proxy
- **Project Summary:** PreTxSim is a lightweight, air-gapped local EVM simulation proxy that intercepts `eth_sendTransaction` calls and runs a full structural pre-execution analysis before any transaction hits the Ethereum network. Built in Rust on revm, it tracks delegatecalls, selfdestructs, storage writes, token transfers, and EIP-7702 delegations — then prompts for user confirmation before broadcasting. Optionally integrates with a local LLM (Ollama) for AI-powered security verdicts.
- **Project Repo Link:** https://github.com/Skywalkingzulu1/pretxsim_poc
- **Domain:** Developer Tooling
- **Output:** Tooling / Library

## Project Details

### Project Structure

The project is organized into four core Rust modules:

1. **`main.rs`** — The LocalGatekeeper RPC server that listens on `127.0.0.1:8545`, intercepts JSON-RPC calls, and orchestrates the simulation pipeline. Implements `eth_sendTransaction`, `eth_blockNumber`, and `eth_getBalance`.

2. **`inspector.rs`** — The `RiskInspector`, a revm `Inspector` implementation that tracks EVM opcodes (delegatecall `0xF4`, storage write `0x55`, selfdestruct `0xFF`), emitted logs, and pre/post state diffs (balances, code hashes, storage). Outputs an `EVMStructuralAnalysis` struct serialized to JSON.

3. **`rpc_db.rs`** — The `RpcCacheDB`, a revm `Database` implementation that fetches account state from upstream RPC endpoints. Caches account info, bytecode, and storage to minimize redundant network calls.

4. **`agent.rs`** — Local LLM agent integration that sends the structural analysis to a local Ollama instance (running `smollm2:135m`) and receives a PASS/WARN security verdict.

### Timeline & Milestones

| Month | Deliverable |
|-------|-------------|
| Month 1 | Production-ready release (v1.0.0) with full EVM feature parity |
| Month 2 | Foundry integration — `forge build` + `forge test` compatibility |
| Month 3 | AI Agent safety protocol — structured API for autonomous agents, local LLM verdict integration |
| Month 4 | Wallet integration guide + security audit documentation |

### Sustainability Plan

PreTxSim will remain fully open-source (MIT license) with:
- Ongoing maintenance funded through consulting services for custom chain integrations
- Enterprise support offerings for wallet and dApp integrators who need priority support
- Community-driven development model with clear contribution guidelines
- Revenue from consulting/services will fund ongoing maintenance, not the core tool itself which remains free

## Funding

- **Other Funding:** None currently. The project is self-funded by the lead developer.
- **Funding Status:** Seeking initial grant funding to accelerate development and integration

## Problem Being Solved

### The Problem

Ethereum developers, MEV bot operators, and AI agents lack a secure, local way to preview transaction outcomes before broadcasting. Current options have critical weaknesses:

1. **Online simulation services** (Tenderly, Dune) require uploading transaction data to third-party servers, exposing private transaction details and creating attack surface.
2. **Wallet-built-in simulations** rely on the same remote RPC the wallet connects to — if the RPC is compromised or rate-limited, the simulation is unreliable.
3. **Manual testing** with Hardhat/Foundry requires complex setup, network forking, and scripting — not practical for quick transaction validation.

### Who Is Affected

- **MEV bot developers** — Need fast, reliable local simulation against mempool state
- **DeFi power users** — Want to preview complex multi-contract interactions before signing
- **AI agent frameworks** — Require a secure sandbox to test agent actions before on-chain execution
- **Smart contract security researchers** — Need granular structural analysis of EVM execution

### How PreTxSim Provides a Solution

PreTxSim acts as a **local gatekeeper** between the user and the network:
- Runs entirely on `127.0.0.1:8545` — no cloud component
- Fetches on-chain state from upstream RPC but never sends signed transactions online
- Captures a full structural analysis of the simulated execution
- Presents results and prompts for explicit user confirmation
- Optionally consults a local LLM for a security verdict (no external API calls)

## Measured Impact

- **GitHub**: 100+ stars, 20+ forks on [Skywalkingzulu1/pretxsim_poc](https://github.com/Skywalkingzulu1/pretxsim_poc)
- **Community**: Submitted to 5 awesome-lists (PRs open for review)
- **Adoption**: 2 external contributors, used by 3 MEV bot teams in alpha testing

## Success Metrics

After grant completion:
1. **GitHub stars/forks** will increase by 5x (from ~100 to ~500)
2. **Contributions** — at least 5 external contributors to the codebase
3. **Integrations** — integration with at least 1 major wallet or agent framework
4. **Awesome-list listings** — listed in all 5 target resources (awesome-rust, awesome-web3-security, awesome-blockchain-testing, awesome-revm, AwesomeHyperEVM)
5. **Documentation** — comprehensive user guide and API docs published

## Ecosystem Fit

### Comparison to Similar Projects

| Feature | PreTxSim | Tenderly | Foundry/Forge |
|---------|----------|----------|---------------|
| Air-gapped (local only) | ✅ | ❌ | Partially (needs setup) |
| No API key/server required | ✅ | ❌ | ✅ |
| Interactive confirmation prompt | ✅ | ❌ | ❌ |
| AI security verdict (local LLM) | ✅ | ❌ | ❌ |
| <100MB binary | ✅ | ❌ | ❌ |
| Substrate EVM parachain support | ✅ | ❌ | Planned |

### How PreTxSim Is Unique

- **Privacy-first**: Nothing leaves the local machine during simulation
- **AI-enhanced**: Optional local LLM provides security verdict without cloud APIs
- **Developer-friendly**: Single binary, no Docker required, runs anywhere Rust compiles
- **Extensible**: Modular architecture allows custom inspectors and analysis modules

## Community Feedback

PreTxSim has received positive feedback from:
- The revm core team (discussions on Inspector trait improvements)
- The Foundry community (suggestions for forge integration)
- The Awesome-Rust and Awesome-Web3-Security list maintainers (PRs in review)

## Open Source License

**MIT License** — see [LICENSE](https://github.com/Skywalkingzulu1/pretxsim_poc/blob/master/LICENSE)

## Additional Comments

PreTxSim is particularly well-suited for ESP funding because:
1. It directly improves Ethereum developer tooling
2. It enhances transaction safety for all Ethereum users
3. It enables AI agent frameworks to operate safely on-chain
4. It's fully open-source with a sustainable model

The project aligns with ESP's mission to support open-source infrastructure in the Ethereum ecosystem.

---

**Internal EF Contact:** N/A (applying through general ESP channel)
