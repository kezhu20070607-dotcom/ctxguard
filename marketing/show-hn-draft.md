# Show HN draft — ctxguard

## Title (pick A/B/C)

**A** — Show HN: ctxguard – ulimit for Claude Code's 200k context window

**B** — I parsed 7 days of my Claude Code sessions and found 2.1B context tokens

**C** — Show HN: ctxguard – a Rust CLI that catches context-window blowups before they cost $1400/wk

**D** *(new)* — Show HN: ctxguard – context-window budgets for Claude Code AND Codex CLI, in 24 ms

## Body (use A or C title; body works with all)

```text
Hi HN,

I built ctxguard after one of my Claude Code sessions hit 558M context tokens
(2790× the 200k window) without me noticing. The wallet wasn't screaming — prompt
caching hides cache_read from your bill — but the model was reading the same
3 files 30 times per turn.

![demo](https://raw.githubusercontent.com/kezhu-20070607/ctxguard/master/demo.gif)

*(30 s terminal recording auto-rendered by CI on every push — refreshes itself)*

ctxguard is a single Rust binary that does three things:

1. `ctxguard profile --days N` — walks ~/.claude/projects/ and shows where
   your context tokens actually go (group by model/day/hour/file).

2. `ctxguard parse <session.jsonl>` — drill into one session's input/output/
   cache_read/cache_write.

3. `ctxguard run --budget=N --on-full=warn|compress|kill -- <cmd>` — wraps a
   live Claude Code run, watches the session JSONL via notify, and fires when
   effective_context crosses the budget. `--on-full kill` is for people who
   learned the hard way (like me).

Real numbers from my last 7 days:
  11 sessions · 2.1B context tokens
  top day:  845M  (2026-07-14)
  top hour: 558M  (one deep-refactor session)
  cache_read accounts for ~95% of every long session

Why Rust: parsed a 14 MB session in 37 ms vs ccusage's 30 572 ms (812× faster
on the same input). Single 1.1 MB binary, zero npm tree.

Differences from existing tools:
  • `effective_context` column (input + cache_read + cache_write) — nobody else
    shows this; it's what actually counts toward your 200k window.
  • Live budget enforcement via JSONL file watch, not a SaaS dashboard.
  • Single binary, no daemon, no Docker.

Install: `cargo install ctxguard` (or grab a binary from GitHub Releases)
Source: https://github.com/kezhu-20070607/ctxguard
Tested on Linux/macOS/Windows (CI runs all three).

Happy to answer questions or take PRs for Codex/Aider adapters.
```

## Submit instructions

1. Go to https://news.ycombinator.com/submit
2. Paste title + body
3. URL: https://github.com/kezhu-20070607/ctxguard
4. Best time: weekday 8-10am US Eastern (peak HN traffic)
