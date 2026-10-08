# PreTxSim demo tape — 35-45s narrative arc, Catppuccin Mocha
# Emits both .mp4 (social feeds) and .gif (GitHub README inline embed).
#
# Run from repo root:   vhs docs/assets/demo.tape
# Build first so no compile chatter appears on camera:
#     cargo build --release
#
# Requires fixtures produced by demo/setup_chain.sh (anvil at 127.0.0.1:8546):
#     demo/tx_risky.json   — DELEGATECALL + SSTORE trigger
#     demo/tx_7702.json    — delegated-EOA target (0xEF0100 designator)
#
# Timeline:
#   00:00-00:10  Boot proxy (air-gapped bind on 127.0.0.1:8545)
#   00:10-00:25  The Trap: eth_sendTransaction hits the proxy
#   00:25-00:45  The Gate: risk panel, LLM brief, interactive [y/N]

Output docs/assets/pretxsim_demo.mp4
Output docs/assets/pretxsim_demo.gif

Set Shell "C:\Program Files\Git\bin\bash.exe"
Set FontSize 16
Set Width 1200
Set Height 700
Set Padding 24
Set Theme "Catppuccin Mocha"
Set TypingSpeed 60ms
Set PlaybackSpeed 1.0

# ---------- 00:00-00:10 | Setup ----------
Hide
Type "export RPC_URL=http://127.0.0.1:8546"
Enter
Type "./target/release/pretxsim_poc &> /tmp/pretxsim.log &"
Enter
Sleep 800ms
Show
Sleep 500ms

Type "cat /tmp/pretxsim.log"
Enter
Sleep 1800ms

# ---------- 00:10-00:25 | The Trap ----------
Type@80ms "curl -s -X POST http://127.0.0.1:8545 -d @demo/tx_risky.json"
Enter
Sleep 4500ms
Ctrl+C

Type "clear"
Enter
Sleep 400ms

Type@80ms "curl -s -X POST http://127.0.0.1:8545 -d @demo/tx_7702.json"
Enter
Sleep 4500ms
Ctrl+C

# ---------- 00:25-00:45 | The Gate ----------
Type "clear"
Enter
Sleep 300ms
Type "# RiskInspector flags delegatecall + EIP-7702 delegation before broadcast"
Enter
Sleep 1500ms
