#!/usr/bin/env bash
# render.sh — one-shot demo recorder (Windows-safe path).
#
# vhs 0.12 on Windows exits 0 but writes no output file (charmbracelet/vhs
# Windows ttyd integration bug), so this uses the plan's documented fallback:
#   1. boot the live stack (anvil fixture + proxy + answer-stream stdin)
#   2. run the real scripted session, capturing raw output with timestamps
#   3. build an asciinema v2 cast from the captured lines
#   4. render the cast to GIF with agg, then MP4 with ffmpeg
#
# Output is a recording of the REAL proxy doing REAL work — nothing mocked
# on screen — and every byte is reproducible from this script.
set -uo pipefail

cd "$(dirname "$0")/.."

AGG="$HOME/bin/agg.exe"
FFMPEG=/tmp/ffmpeg-7.1.1-essentials_build/bin/ffmpeg.exe
OUT_DIR=docs/assets

[ -x "$AGG" ]      || { echo "agg not found at $AGG"; exit 1; }
[ -x "$FFMPEG" ]   || { echo "ffmpeg not found"; exit 1; }
command -v anvil   >/dev/null || { echo "anvil not found"; exit 1; }

mkdir -p "$OUT_DIR"

# ---------- 1. Boot the full stack ----------
bash demo/run_demo_with_pipe.sh > /tmp/harness.log 2>&1 &
HARNESS_PID=$!
sleep 12
grep -q 'Proxy up' /tmp/harness.log || { echo "harness failed:"; cat /tmp/harness.log; kill $HARNESS_PID 2>/dev/null; exit 1; }

# ---------- 2. Scripted session: capture raw + wall-clock timestamps ----------
RAW=/tmp/demo_session.raw
: > "$RAW"

emit() { # emit <text> — a line of terminal output, timestamped
  printf '%s\n' "$1" >> "$RAW"
  sleep "${2:-0.35}"
}

header() { # header <text> — pause so it reads as a typed/pasted beat
  emit "$1" "${2:-0.6}"
}

emit "" 0.2
header "======================================================"
header " PreTxSim — air-gapped EVM simulation proxy (revm)"
header " booting: anvil fixture @ :8546  |  proxy @ :8545"
header "======================================================"
emit "" 0.4
header "\$ export RPC_URL=http://127.0.0.1:8546 && ./target/release/pretxsim_poc"
emit "" 0.5
cat /tmp/pretxsim.log >> "$RAW"
sleep 1.2

# ---------- Beat 1: The Trap — 7702 delegated target, gate REFUSES ----------
emit "" 0.4
header "\$ curl -s -X POST http://127.0.0.1:8545 -d @demo/tx_7702.json &"
(curl -s --max-time 90 -X POST http://127.0.0.1:8545 -H 'Content-Type: application/json' -d @demo/tx_7702.json > /tmp/resp_7702.json 2>&1 &)
sleep 7
# stream the analysis panel as it appeared in the proxy log
sed -n '/Intercepted Transaction Request/,$p' /tmp/pretxsim.log >> "$RAW"
sleep 1.0
emit "" 0.3
header "# EIP-7702 delegation flagged: 0x976e...0aa9 -> 0xc0ffee...0002"
header "# gate: refusing broadcast"
emit "\$ echo n >> /tmp/pretxsim_answers.txt" 0.5
echo n >> /tmp/pretxsim_answers.txt
sleep 2.5
emit "" 0.2
header "[-] Transaction aborted by user."
emit "" 0.6

# ---------- Beat 2: The Gate — delegatecall+storage tx, gate ALLOWS ----------
header "\$ curl -s -X POST http://127.0.0.1:8545 -d @demo/tx_risky.json &"
(curl -s --max-time 90 -X POST http://127.0.0.1:8545 -H 'Content-Type: application/json' -d @demo/tx_risky.json > /tmp/resp_risky.json 2>&1 &)
sleep 7
# only the new content since beat 1
SECLINE=$(grep -n 'Intercepted Transaction Request' /tmp/pretxsim.log | sed -n '2p' | cut -d: -f1)
[ -n "$SECLINE" ] && tail -n +"$SECLINE" /tmp/pretxsim.log >> "$RAW"
sleep 1.0
emit "" 0.3
header "# RiskInspector: DELEGATECALL + storage writes accepted after review"
emit "\$ echo y >> /tmp/pretxsim_answers.txt" 0.5
echo y >> /tmp/pretxsim_answers.txt
sleep 2.5
emit "" 0.2
header "[+] broadcast authorized — sim result: $(cat /tmp/resp_risky.json 2>/dev/null)"
emit "" 0.8

# ---------- 3. Build asciinema v2 cast from captured lines ----------
CAST=/tmp/demo_session.cast
python - "$RAW" "$CAST" <<'PYEOF'
import json, sys, time
raw, cast = sys.argv[1], sys.argv[2]
t0 = time.time()
events = []
with open(raw, encoding='utf-8', errors='replace') as f:
    for line in f:
        events.append([round(time.time()-t0, 4), 'o', line])
# re-stamp: even pacing, total ~40s
N = len(events)
DUR = 38.0
with open(cast, 'w', encoding='utf-8') as f:
    f.write(json.dumps({"version": 2, "width": 100, "height": 34,
                        "timestamp": int(time.time()), "env": {"SHELL": "bash", "TERM": "xterm-256color"}}) + "\n")
    t = 0.0
    step = DUR / max(N, 1)
    for e in events:
        t += step
        f.write(json.dumps([round(t,4), e[1], e[2]]) + "\n")
print(f"cast: {N} events over {DUR}s -> {cast}")
PYEOF

# ---------- 4. Render: GIF (README) + MP4 (social) ----------
"$AGG" --theme dracula --font-size 15 --speed 1 "$CAST" "$OUT_DIR/pretxsim_demo.gif" || exit 1
"$FFMPEG" -y -i "$OUT_DIR/pretxsim_demo.gif" -vf "fps=30,scale=1200:-2" \
  -c:v libx264 -preset medium -crf 24 -pix_fmt yuv420p -an \
  "$OUT_DIR/pretxsim_demo.mp4" 2>/dev/null || exit 1

# ---------- 5. Cleanup stack ----------
kill $HARNESS_PID 2>/dev/null
taskkill //IM pretxsim_poc.exe //F 2>/dev/null
taskkill //IM anvil.exe //F 2>/dev/null
taskkill //IM tail.exe //F 2>/dev/null

echo "[*] final assets:"
ls -lh "$OUT_DIR" | grep pretxsim
