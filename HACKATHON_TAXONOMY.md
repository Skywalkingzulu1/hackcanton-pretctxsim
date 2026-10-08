# PreTxSim — Hackathon Targeting Taxonomy (October 2026)

> **12 qualifying events** · 7 primary Web3/DeFi competitions + 5 cross-over Agentic AI / DevSecOps / FinTech challenges
> Target project: **PreTxSim** — air-gapped local EVM simulation proxy (Rust, `revm`) that intercepts `eth_sendTransaction`, runs full structural pre-execution analysis, optionally consults a local Ollama agent for a PASS/WARN verdict, and gates broadcast behind interactive confirmation.
> Data cut: 2026-10-07. Re-verify each event's official rules (tracks, banned stacks, eligibility, IP clauses) before submitting.

---

## 0. The Product-in-One-Liner (pitch opener)

> **"PreTxSim simulates your transaction before the network ever sees it — a local, air-gapped EVM rehearsal with a risk inspector and an offline AI auditor sitting between your wallet and the mempool."**

## 1. The Available Trojan Horses (one hook per event)

Per the Winning Hackathon Taxonomy rule — *one Trojan Horse > ten features* — each submission picks exactly ONE of these as its centerpiece:

| # | Asset | What it proves |
|---|-------|----------------|
| A | **RiskInspector** | Flags delegatecalls, selfdestructs, storage writes, allowance changes, token transfers, **EIP-7702 delegations** |
| B | **revm deterministic simulation** | Full pre-execution replay against live RPC state — balance deltas, gas, slippage, condition outcomes |
| C | **Local LLM auditor (Ollama `smollm2`)** | Deterministic checks + AI soft-check separation; PASS/WARN brief with zero data leaving the machine |
| D | **The Gate** | Interactive refuse/accept before broadcast — security guardrails at the exact signing moment |
| E | **Air-gapped / <800MB** | Private keys and state never exposed to remote services — enterprise & compliance posture |
| F | **RPC proxy form factor** | Drop-in on `127.0.0.1:8545` — zero integration cost for wallets, bots, agents, CI pipelines |

---

## 2. Tier 1 — Primary Web3 & Smart Contract Competitions (7 Events)

| Event | Deadline | Track Focus | PreTxSim Trojan Horse | Pitch Angle |
| --- | --- | --- | --- | --- |
| **Colosseum (Crypto World's Fair)** | Oct 12, 2026 | Developer Tooling & Security: EVM tracing, `eth_call` state overrides, user-facing guardrails (Ethereum + Base) | A + B + D | Ship the simulation layer as tooling devs point their wallet at — "state overrides without the RPC round-trip risk" |
| **Monad Metropolis Hackathon** | Oct 13, 2026 | Onchain Finance & Security: low-latency tx outcomes, balance deltas, slippage on EVM-compatible execution layers | B | Rehearse the tx on Monad-compatible state pre-flight; quantify balance delta + slippage before signing |
| **ETH Lagos 2026** | Oct 10, 2026 | Ethereum Infrastructure & dApps: execution simulation, honeypot detection, wallet-level safety | A + D | Live honeypot/drain-pattern demo: risky tx refused at the gate in front of judges |
| **BNB Hack: Tokenized Stocks Edition** | Oct 11, 2026 | RWA & Trading Tooling: validate state changes, asset transfers, approval limits pre-swap | A (allowance-change inspector) | "Simulate the approval + transfer chain of a tokenized-stock swap before any share moves" |
| **BLI Legal Tech Hackathon 2** | Oct 31, 2026 | RegTech & Compliance: pre-execution verification against compliance policy | D + E | Policy-as-rules engine: contract calls are checked against compliance policy before signing; audit trail stays local |
| **Event Contracts Hackathon** | Oct 31, 2026 | Web3 Event Tooling: simulate state changes, gas costs, condition triggers for event-driven contracts | B | Re-run event-triggered calls against current state: does the condition fire, at what gas, with what side effects? |
| **From Vision to Impact** | Oct 24, 2026 | DAO & Governance Infrastructure: validate multi-sig proposals before mainnet broadcast | B + D | Simulate every proposed execution of a multisig/DAO proposal; each signer gets the risk brief before approving |

---

## 3. Tier 2 — Cross-Over: Agentic AI, DevSecOps & FinTech (5 Events)

| Event | Deadline | Positioning | Adaptation Required |
| --- | --- | --- | --- |
| **TUM Blockchain & AI Hackathon** | Oct 31, 2026 | **Agentic payment safety rails** — mandatory sandbox simulation for autonomous AI agents executing on-chain payments (prevents unintended token drains) | Lean on `src/agent.rs` (Ollama integration): demo an agent whose tx is auto-simulated + gated — the agent *asks the local auditor* before acting |
| **Life After Code by GitLab** | Oct 27, 2026 | **Web3 DevSecOps pipeline** — pre-deployment/pre-execution security testing inside smart-contract CI/CD | Add a CI job example: `pretxsim --ci` runs simulation + RiskInspector assertions as a pipeline stage (fails build on flagged tx patterns) |
| **Zecathon 6.0** | Oct 13, 2026 | **Financial security & anti-fraud** — protect users and banking gateways from drainers and malicious contract logic | Frame RiskInspector signatures as a "drainer signature database": known drain patterns refused pre-broadcast |
| **European AI Hackathon (Web3 Track)** | Oct 29, 2026 | **Enterprise Web3 security** — state simulation + contract safety for institutional dApps | Emphasize E (air-gapped) + C (local LLM): no enterprise tx data leaves the VPC |
| **#HackCanton Season 3** | Oct 9, 2026 | **Enterprise privacy & contract logic** — verify privacy-preserving executions and enterprise state transitions | Show simulation of state transitions against private/testnet state without exposing keys or submitting to a public mempool |

---

## 4. Priority Queue (deadline-ordered, today = Oct 7)

| When | Event | Action |
| --- | --- | --- |
| **Oct 9** | #HackCanton Season 3 | Earliest deadline — minimal adaptation, submit fast |
| **Oct 10** | ETH Lagos 2026 | Flagship demo: honeypot refuse-at-gate |
| **Oct 11** | BNB Hack (Tokenized Stocks) | Allowance/transfer pre-validation demo |
| **Oct 12** | **Colosseum** | **Highest prize ceiling — prioritize polish + tooling pitch** |
| **Oct 13** | Monad Metropolis + Zecathon 6.0 | Same core demo, two narratives (speed/slip vs. anti-fraud) |
| Oct 24–31 | Remaining 6 events | Reuse the same core; adapt only the pitch layer |

---

## 5. Reusable Submission Kit (build once, submit 12×)

Distilled from the Winning Hackathon Taxonomy rules (D3/D4):

- [ ] **README**: award/track badge line at very top, live demo link first, one architecture diagram (ASCII/Mermaid), demo GIF (`docs/assets/pretxsim_demo.gif` — already exists)
- [ ] **`DEMO_RUNBOOK.md`**: exact steps a judge can replay (anvil fixture + proxy + flagged tx + gate refusal)
- [ ] **Quantified proof**: e.g. simulation overhead in ms, RAM measured, # of RiskInspector categories, EIP-7702 flag case
- [ ] **Per-event branch**: `pretxsim@<event>` — only the pitch doc + one integration glue file differ per event
- [ ] **Deterministic + LLM separation** stated explicitly (structural inspector = provable, Ollama = advisory) — matches the judged rubric pattern from Vergil/ONCHOR winners
- [ ] **Deployment config as first-class deliverable**: `docker-compose.yml` for proxy + Ollama so judges run it in one command

---

*Companion to: `Skywalkingzulu1/hthon-intel` (organizers intel, sponsor strategy, judges, winning taxonomy).*
