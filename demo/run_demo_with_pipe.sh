#!/usr/bin/env bash
# run_demo_with_pipe.sh — make the proxy's interactive [y/N] gate drivable
# from the VHS tape.
#
# Windows/MSYS named pipes (mkfifo) do NOT deliver writes to a process
# reading stdin via read_line — writes hang. Instead we use the portable
# `tail -f <answer-file> | proxy` pattern: the tape appends `y` or `n` to
# the answer file and tail streams it into the proxy's stdin. Identical
# on-camera behavior, works everywhere (Linux CI included).
set -uo pipefail

cd "$(dirname "$0")/.."

ANSWERS=/tmp/pretxsim_answers.txt
LOG=/tmp/pretxsim.log

# 1. Chain fixture (anvil + injected bytecode) — idempotent
bash demo/setup_chain.sh

# 2. Answer stream: tail -f keeps stdin open, tape appends y/n to it
: > "$ANSWERS"
rm -f "$LOG"

# 3. Boot the proxy with stdin wired to the answer stream
RPC_URL=http://127.0.0.1:8546 bash -c 'tail -f /tmp/pretxsim_answers.txt | ./target/release/pretxsim_poc' > "$LOG" 2>&1 &
TAILER_PID=$!

cleanup() {
  kill "$TAILER_PID" 2>/dev/null || true
  # kill the whole pipeline (tail + proxy)
  ps -ef 2>/dev/null | grep -E 'pretxsim_poc|tail -f /tmp/pretxsim_answers' | grep -v grep | awk '{print $2}' | xargs -r kill 2>/dev/null || true
  taskkill //IM pretxsim_poc.exe //F 2>/dev/null || true
}
trap cleanup EXIT

sleep 1
echo "[+] Proxy up on 127.0.0.1:8545, log: $LOG"
echo "[+] Trigger a tx:    curl -s -X POST http://127.0.0.1:8545 -d @demo/tx_7702.json"
echo "[+] Answer the gate: echo y >> $ANSWERS"

wait "$TAILER_PID" 2>/dev/null || true
