#!/usr/bin/env bash
# setup_chain.sh — deterministic anvil fixture for the PreTxSim demo.
#
# Boots anvil on 127.0.0.1:8546 with a fixed mnemonic, deploys the Risky
# fixture (real SSTORE + DELEGATECALL) and plants an EIP-7702 delegation
# designator (0xEF0100 || delegatee) on a fixed EOA via anvil_setCode.
#
# Addresses are fixed, matching demo/tx_risky.json and demo/tx_7702.json:
#   Risky contract : 0xc0ffee0000000000000000000000000000000001
#   Delegatee      : 0xc0ffee0000000000000000000000000000000002
#   Delegated EOA  : 0x976ea74026e726554db657fa54763abd0c3a0aa9  (anvil acct #1)
set -euo pipefail

cd "$(dirname "$0")"

ANVIL_URL=http://127.0.0.1:8546
RISKY=0xc0ffee0000000000000000000000000000000001
DELEGATEE=0xc0ffee0000000000000000000000000000000002
DELEGATED_EOA=0x976ea74026e726554db657fa54763abd0c3a0aa9

command -v anvil >/dev/null || { echo "anvil not found"; exit 1; }
command -v forge >/dev/null || { echo "forge not found"; exit 1; }
command -v cast   >/dev/null || { echo "cast not found";   exit 1; }

# 1. Boot anvil (caller may have started it already; tolerate either way)
if curl -s --max-time 2 "$ANVIL_URL" -X POST -H 'Content-Type: application/json' \
     -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' >/dev/null 2>&1; then
  echo "[*] anvil already running on $ANVIL_URL"
else
  anvil --port 8546 > /tmp/anvil.log 2>&1 &
  echo "[*] started anvil (pid $!) on $ANVIL_URL"
  for i in $(seq 1 30); do
    curl -s --max-time 1 "$ANVIL_URL" -X POST -H 'Content-Type: application/json' \
      -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' >/dev/null 2>&1 && break
    sleep 0.3
  done
fi

# 2. Build fixtures (skip if artifacts already exist)
if [ ! -f contracts/out/Risky.sol/Risky.json ]; then
  echo "[*] building fixture contracts..."
  (cd contracts && forge build)
fi

RISKY_RUNTIME=$(python -c "import json;print(json.load(open('contracts/out/Risky.sol/Risky.json'))['deployedBytecode']['object'])")
DELEGatee_RUNTIME=$(python -c "import json;print(json.load(open('contracts/out/Risky.sol/Delegatee.json'))['deployedBytecode']['object'])")

# 3. Inject runtime bytecode at fixed addresses
cast rpc --rpc-url "$ANVIL_URL" anvil_setCode "$RISKY" "$RISKY_RUNTIME" >/dev/null
cast rpc --rpc-url "$ANVIL_URL" anvil_setCode "$DELEGATEE" "$DELEGatee_RUNTIME" >/dev/null
echo "[*] Risky + Delegatee bytecode injected"

# 4. Plant the EIP-7702 designator on the delegated EOA: 0xEF0100 || delegatee
DESIGNATOR="0xef0100${DELEGATEE#0x}"
cast rpc --rpc-url "$ANVIL_URL" anvil_setCode "$DELEGATED_EOA" "$DESIGNATOR" >/dev/null
echo "[*] 7702 designator planted on $DELEGATED_EOA -> $DELEGATEE"

# 5. Verify injection round-trips (proves rpc_db standard-methods refactor works)
GOT=$(cast code --rpc-url "$ANVIL_URL" "$RISKY")
[ "${#GOT}" -gt 4 ] || { echo "FAILED: Risky code empty"; exit 1; }
GOT7702=$(cast code --rpc-url "$ANVIL_URL" "$DELEGATED_EOA")
[ "$GOT7702" = "$DESIGNATOR" ] || { echo "FAILED: designator mismatch: $GOT7702"; exit 1; }

echo "[+] chain fixture ready: $ANVIL_URL"
