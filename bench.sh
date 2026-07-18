#!/usr/bin/env bash
# bench.sh — measure ctxguard vs ccusage on real Claude Code JSONL
#
# Usage: ./bench.sh <path-to-session.jsonl>
#
# Reports wall time and peak RSS for both tools on the same input.

set -e

if [ -z "$1" ] || [ ! -f "$1" ]; then
    echo "Usage: $0 <path-to-session.jsonl>"
    echo "Tip: pick a session >= 100 MB for meaningful numbers"
    exit 1
fi

JSONL="$1"
SIZE=$(du -h "$JSONL" | cut -f1)
echo "=== ctxguard vs ccusage benchmark ==="
echo "input:    $JSONL ($SIZE)"
echo

# Make sure ccusage is available
if ! command -v npx >/dev/null 2>&1; then
    echo "skipping ccusage: npx not found"
    CTXGUARD_ONLY=1
fi

run_bench() {
    local tool="$1"
    local cmd="$2"
    echo "--- $tool ---"
    /usr/bin/time -f "wall=%es rss=%MKB" $cmd 2>&1 | tail -1 || true
    echo
}

run_bench "ctxguard parse"   "./target/release/ctxguard parse $JSONL > /dev/null"

if [ -z "$CTXGUARD_ONLY" ]; then
    run_bench "ccusage daily"   "npx -y ccusage@latest daily --json $JSONL > /dev/null"
    run_bench "ccusage session" "npx -y ccusage@latest session --json $JSONL > /dev/null"
fi
