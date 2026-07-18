# ctxguard

> Context-window budget enforcer for AI coding agents — `ulimit` for Claude Code, Codex, and Aider.

## Why

A typical 30-minute Claude Code session re-reads `auth.ts` 30 times. prompt caching hides this
from your wallet but **not** from your context window. One session can quietly burn
**100M+ context tokens** — and the next model load fails with "context window exceeded" mid-task.

`ctxguard` is the missing safety net. Parse past sessions to see where tokens go, and (W2) wrap
live agent runs with a hard budget that triggers `warn | compress | kill` when you cross it.

```
$ ctxguard parse ~/.claude/projects/<proj>/<session>.jsonl
file:        C:/Users/zk/.claude/projects/.../cd3b58f7.jsonl
model:       claude-opus-4-8
turns:       128
first / last: 2026-07-18T12:01:25Z / 2026-07-18T12:52:46Z  (51 min)
input:       479081
output:      102575
cache_read:  11861184
cache_write: 0
---
total billed:    12442840
effective ctx:   12340265     ← what really counts toward the 200k window
```

```
$ ctxguard profile --days 7
| session       | model         | turns | input   | output | cache_rd | ctx_window |
|---------------|---------------|-------|---------|--------|----------|------------|
| 7a811832.jsonl| claude-opus-4-8|  514  |  2.5M   | 216.0K |  71.3M   |  73.8M     |
| f744bd4b.jsonl| claude-opus-4-8|  206  | 873.5K  | 142.8K |  23.4M   |  24.3M     |
| 4f78d3af.jsonl| claude-opus-4-8|  687  | 762.8K  | 183.1K | 107.1M   | 107.9M     |

11 sessions  ·  total billed: 612M  ·  effective context: 540M
```

## Install

```bash
cargo install ctxguard
# or
brew install ctxguard   # coming soon
```

[![CI](https://github.com/zhuke-ai/ctxguard/actions/workflows/ci.yml/badge.svg)](https://github.com/zhuke-ai/ctxguard/actions)

## Usage

```bash
# Inspect a past Claude Code session
ctxguard parse ~/.claude/projects/<proj>/<session>.jsonl

# Aggregate token usage across your last week of work
ctxguard profile --days 7

# Wrap a live agent run with a hard budget
ctxguard run --budget 80000 --on-full warn -- claude "fix the auth bug"
ctxguard run --budget 80000 --on-full compress -- claude "refactor module X"
ctxguard run --budget 80000 --on-full kill -- claude "try everything"
```

### Why three modes?

| mode | what happens | when to use |
|---|---|---|
| `warn` | print to stderr, child keeps running | you just want visibility |
| `compress` | send `/compact` to child's stdin | you want it to keep working but trim context |
| `kill` | SIGTERM the child | you've hit your wallet limit for the day |

## How it's different

- **`effective_context`** — the column nobody else shows. Sum of `input_tokens + cache_read +
  cache_creation`. This is what your model actually loads, not what Anthropic/OpenAI bills.
- **Single binary, no daemon** — `cargo install` and you're done. No npx, no Docker, no SaaS.
- **Local-first** — reads `~/.claude/projects/` directly. Your session JSONL never leaves your
  machine.

## Roadmap

- [x] **W1** — `parse` and `profile` subcommands. Token aggregation across sessions.
- [x] **W2** — `ctxguard run --budget=N --on-full=warn|compress|kill -- <cmd>`. File-watcher + budget trigger.
- [ ] **W3** — `ctxguard profile --by tool|hour|model` to break down where tokens go.
- [ ] **W4** — Codex + Aider adapters (right now we only parse Claude Code JSONL).

## Benchmarks (real, W3)

Single 14 MB Claude Code session JSONL on Windows 11 / Node 24 / Rust 1.94:

| tool | operation | wall time | binary size | dependency footprint |
|---|---|---|---|---|
| `ctxguard` | `parse <file>` | **37 ms** | 1.1 MB | zero (single binary) |
| `ctxguard` | `profile --days 7` (all sessions) | **451 ms** | 1.1 MB | zero |
| `ccusage` | `daily --json` (one project) | **30 572 ms** | 38 MB (Node + npm tree) | 247 packages |

That's **~810× faster than `ccusage`** on a representative workload — Rust startup
+ `memmap2` zero-copy parsing + single-pass serde_json deserialization vs Node.js
cold-start + V8 GC + JSON.parse on the whole file. The gap widens as your session
history grows.

Run it yourself: `./bench.sh <path-to-session.jsonl>`.

## License

MIT OR Apache-2.0

## Contributing

Run the local test:
```bash
# Terminal 1
./target/release/ctxguard run --budget 5000 --on-full warn --session /tmp/test.jsonl -- sleep 60

# Terminal 2
python tests/mock_session.py /tmp/test.jsonl --turns 10 --step-cache-read 1000
```

You should see `[ctxguard] BUDGET HIT` after ~5 turns.
