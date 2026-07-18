# Example: Budget a Claude Code refactor session

Goal: keep a multi-hour refactor under 200k context tokens so it never blows
the model load, without paying the cache_read tax.

## Setup

```bash
# Install
cargo install ctxguard
# or:  curl -fsSL https://raw.githubusercontent.com/kezhu-20070607/ctxguard/master/install.sh | sh
```

## Run with budget

```bash
ctxguard run --budget 150000 --on-full compress -- claude "refactor auth.ts to use the new JWT lib"
```

What happens:

1. `ctxguard` spawns `claude` as a child process, inheriting stdio.
2. It locates the freshly-created session JSONL in `~/.claude/projects/<cwd-hash>/`.
3. It opens a `notify` watcher on that directory.
4. Every time the JSONL gets a new assistant turn (every few seconds in a long
   refactor), it re-parses the file and sums `effective_context`.
5. When cumulative context crosses 150 000, it sends `/compact` to the child's
   stdin. Claude Code's `/compact` summarises the conversation in-place.
6. The refactor continues, and you stay inside the budget.

## Three policies

| policy | when to use it |
|---|---|
| `--on-full warn` | exploratory sessions — you want a heads-up, not a stop |
| `--on-full compress` | long-running refactors — keep working, trim context |
| `--on-full kill`   | cost-conscious sessions — never spend more than $X per day |

## Inspect the cost of a finished session

```bash
ctxguard profile --days 7 --by day
```

Output:

```
effective context by day (top 20):
day                              ctx_tokens       %
2026-07-14                        845.6M      39%
2026-07-11                        404.7M      19%
...
total: 2.1B across 7 day buckets
```

That 2.1 B is what your 200k window is *actually* fighting against, day after day.

## Combine with other tools

```bash
# pipe JSON output to jq for further analysis
ctxguard profile --days 30 --by hour | tail -20

# compare to ccusage to triangulate cost
npx -y ccusage@latest daily
```

## See also

- [SKILL.md](../ctxguard-skill.md) — make ctxguard available to Claude Code itself
- [../bench.sh](../bench.sh) — benchmark ctxguard vs ccusage on your own sessions
