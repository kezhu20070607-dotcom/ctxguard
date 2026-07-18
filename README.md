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

## How it's different

- **`effective_context`** — the column nobody else shows. Sum of `input_tokens + cache_read +
  cache_creation`. This is what your model actually loads, not what Anthropic/OpenAI bills.
- **Single binary, no daemon** — `cargo install` and you're done. No npx, no Docker, no SaaS.
- **Local-first** — reads `~/.claude/projects/` directly. Your session JSONL never leaves your
  machine.

## Roadmap

- [x] **W1** — `parse` and `profile` subcommands. Token aggregation across sessions. *← you are here*
- [ ] **W2** — `ctxguard run --budget=80k --on-full=warn claude "fix bug"`. Real-time enforcement.
- [ ] **W3** — `ctxguard profile --by tool` to break down where cache_read comes from.
- [ ] **W4** — Codex + Aider adapters (right now we only parse Claude Code JSONL).

## License

MIT OR Apache-2.0
