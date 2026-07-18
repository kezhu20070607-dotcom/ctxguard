---
name: ctxguard
description: |
  Use ctxguard to inspect or constrain Claude Code's context-window token usage.
  Invoke when the user asks "how much context am I using?", "is this session burning tokens?",
  "set a budget for this run", or wants to compare past sessions by model/day/hour/file.
triggers:
  - "ctxguard"
  - "context window usage"
  - "token budget"
  - "how much context am I using"
  - "are we burning tokens"
exclude_triggers:
  - "don't use ctxguard"
  - "skip ctxguard"
allowed-tools: Bash(ctxguard:*)
---

# ctxguard skill

A local CLI for inspecting Claude Code session token usage and enforcing a context budget
in real time. Single binary, zero config, reads `~/.claude/projects/` directly.

## When to use

| User intent | Command |
|---|---|
| "show me my recent token usage" | `ctxguard profile --days 7` |
| "which day burned the most context" | `ctxguard profile --days 30 --by day` |
| "how big was that session I just ran" | `ctxguard parse ~/.claude/projects/<proj>/<sid>.jsonl` |
| "cap this run at 80k tokens" | `ctxguard run --budget 80000 --on-full warn -- claude "<request>"` |
| "auto-compress when context fills" | `ctxguard run --budget 80000 --on-full compress -- claude "<request>"` |
| "kill the run if I burn too much" | `ctxguard run --budget 80000 --on-full kill -- claude "<request>"` |

## Why this exists

A typical 30-min Claude Code session re-reads `auth.ts` 30 times. Prompt caching hides this
from your wallet but **not** from your context window. Real numbers from 7 days of one user's
history:

- **2.1 billion** effective context tokens across 11 sessions
- top single session: **558M** context tokens (2790× the 200k standard window)
- cache_read accounts for ~95% of every long session

## Install (if not present)

```bash
cargo install ctxguard
# or grab a binary from https://github.com/kezhu-20070607/ctxguard/releases
```

## Notes for Claude

- `ctxguard parse <file>` is safe and fast — run it freely when the user asks.
- `ctxguard profile --days N` walks `~/.claude/projects/`; do not pass arbitrary paths.
- `ctxguard run` spawns a child process and inherits stdio — only invoke when the user has
  explicitly asked to wrap a command. Default budget: 80000 unless the user said otherwise.
- When showing output, highlight the `effective_context` column over `cache_read` alone —
  that's the column nobody else shows.
