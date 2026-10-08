#!/usr/bin/env bash
set -euo pipefail

# PreTxSim Grant Application Video Script
# Usage: Record your screen while running this script, or use it as a voice-over guide

VIDEO_DIR="./funding/videos"
mkdir -p "$VIDEO_DIR"

cat << 'SCRIPT'
PreTxSim — Grant Application Video Script
=========================================

[Opening - 0:00]
"Hi, I'm [Name], lead developer of PreTxSim. Today I'm showing you how our air-gapped local EVM simulation proxy is transforming transaction safety for Ethereum developers, MEV bots, and AI agents."

[Demo Start - 0:10]
"First, let me build and run the proxy."
$ cargo build --release
$ export RPC_URL="https://eth.llamarpc.com"
$ ./target/release/pretxsim_poc

"As you can see, it starts on 127.0.0.1:8545, connecting to our upstream RPC."

[Feature 1: Intercept - 0:30]
"Now, let me simulate a transaction. When an application sends an eth_sendTransaction call to PreTxSim, it intercepts it."
$ curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_sendTransaction",
    "params": [{
      "from": "0x...",
      "to": "0x...",
      "data": "0x..."
    }],
    "id": 1
  }'

[Feature 2: Analysis - 0:45]
"PreTxSim runs the transaction through a full structural analysis using our RiskInspector. It tracks delegatecalls, selfdestructs, storage writes, and token transfers."

"The output shows exactly what would happen on-chain:"
  - Gas estimate
  - State changes (balances, storage, allowances)
  - Any detected risks

[Feature 3: AI Agent - 1:00]
"Then, it consults a local LLM agent via Ollama to provide a security verdict — all completely offline, no API keys needed."

[Feature 4: Confirmation - 1:15]
"Finally, PreTxSim prompts for your confirmation before broadcasting."

[Use Cases - 1:30]
"This is particularly valuable for:
1. MEV bot developers who need fast, local simulation
2. DeFi users protecting against scams
3. AI agents that need a safe sandbox for testing
4. Security researchers auditing contracts"

[Closing - 1:45]
"PreTxSim is open-source on GitHub at Skywalkingzulu1/pretxsim_poc. We're seeking grant funding to accelerate integration with wallets and agent frameworks. Thank you for your consideration."

SCRIPT

echo "Video script saved to $VIDEO_DIR/grant_video_script.sh"
echo "Run: record your screen while executing the demo commands"
