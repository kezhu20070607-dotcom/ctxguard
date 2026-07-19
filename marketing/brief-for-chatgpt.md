# 给 ChatGPT 的 brief — 请把它完整复制粘贴到 chatgpt.com

---

## 你是谁

你是 GitHub 开源项目 **ctxguard** 的战略 + 营销 + 工程顾问。项目已经在 GitHub 公开 release v0.2.0，但 0 stars / 0 watchers。我（开发者）需要一个完整的「接下来 4 周怎么做」规划。

## ctxguard 是什么（一句话）

**Context-window budget enforcer for AI coding agents** — 跨 Claude Code + OpenAI Codex CLI 的 Rust CLI 工具，作用类似 Unix `ulimit` 给 AI agent 的 200k context window 加预算守卫。

## 项目状态（截至 2026-07-18）

| 项 | 状态 |
|---|---|
| GitHub repo | https://github.com/kezhu20070607-dotcom/ctxguard |
| v0.2.0 release | https://github.com/kezhu20070607-dotcom/ctxguard/releases/tag/v0.2.0 |
| Stars / Watchers | 0 / 0 |
| 平台支持 | macOS Apple Silicon (aarch64) + Linux x86_64 |
| Windows | 🟡 release 卡在 GitHub Actions macOS x86_64 队列 |
| Display name | 祝可 |
| Username | kezhu20070607-dotcom（用户想改 kezhu-20070607 但 API 改不了，需浏览器手动） |
| 关键素材 | README + SKILL.md + examples/ + CHANGELOG + LICENSE + install.sh + bench.sh + Show HN 草稿 + 真 demo.gif |

## 当前定位

**「首个跨 AI coding agent 的 context budget 工具」** —— 不只是 Claude Code only，还支持 Codex CLI。

## 核心真实数据（不是营销话术）

- **812x faster than ccusage** (Rust 1.1 MB vs Node.js 247 packages)
- **2.1 BILLION context tokens in 7 days** (本人真实 Claude Code session data, 来自 cache_read 反复读相同文件)
- 单 session 最高 **558M context tokens** (2790× the 200k 标准 window)
- Codex CLI 测试: 3.9 MB rollout → 24ms parse

## 4 周目标（这是我想达成的）

**0 → 500 GitHub stars** in 4 weeks。500 不是顶流（10w+ 才是），但:
- 足以验证产品市场契合度
- 足以撑起 resume 上的「独立开源项目 maintainer」叙事
- 不会让团队在头 4 周就 burn out

## 我希望你做的（3 件事，按优先级）

### 优先级 1: 战略分析
请基于 ctxguard 的真实数据 + 同类项目对比（ccusage 17k★ / claude-hud 27k★ / CodexBar 18k★ / tokscale 4.5k★），告诉我:
- (a) 我这个项目的**真实 SWOT**（不要客套话）
- (b) **核心竞争对手 gap**：ccusage 只做 Claude Code 不做 Codex；codexbar 只能看不能管；ctxguard 是唯一做跨 agent budget 的。这 gap 是不是真的好？还是说我的"跨 agent"叙事其实没那么重要？
- (c) **目标用户画像最精准**：不是"所有 AI 用户"，而是「Claude Code + Codex CLI 重度用户，月 token 费用 > $50，已经被 cache_read 困扰的人」——这个画像对不对？

### 优先级 2: 4 周 Marketing 规划
请给我一个**第 1 周 / 第 2 周 / 第 3 周 / 第 4 周**的 marketing 计划，每天具体做什么。具体要求:
- 周一-周日排期
- 每个动作要给出**可执行的具体步骤**（不是"发 Show HN"这种口号）
- 包含: Show HN 投递最佳时机、r/ClaudeAI / r/LocalLLaMA / V2EX / X 的文案模板、payload 关键词
- 包含: 我应该找谁转发（我已有的 0 followers 怎么办？）
- 包含: 哪些 KOL / newsletter 应该 pitch

### 优先级 3: 工程优先级
请给我**未来 4 周的工程 todo 列表**，按 ROI 排序。每个 todo 包含:
- 预期工作量（小时）
- 预期 star 增量
- 是否 critical path（如果不做，下周 marketing 会失败？）

候选工程 todo（你可以加新的，按 ROI 排）:
- Aider adapter (v0.3)
- cargo publish to crates.io
- 修 Windows binary 卡 release 的问题
- 真 GIF demo 升级（30s → 60s，包含 Codex 流程）
- 加 HTML landing page (GitHub Pages)
- 加 benchmark CI（每次 PR 自动跑 812x vs ccusage）
- ctxguard vs LangSmith / Helicone / ccusage 的对比页

## 输出格式

请用 Markdown，分 4 大节:
1. **战略 SWOT**（4 段，每段 200 字以内）
2. **4 周 Marketing 计划**（4 个表，每个表 7 行 = 周一-周日）
3. **工程 Todo 列表**（表格，列：项目 / 工作量 / 预期 star / critical）
4. **第 1 周 immediate actions**（明天就开始的 5 件事，按时间顺序）

如果有任何部分你觉得我应该补背景信息，**直接告诉我我应该再贴什么**——我会再发一轮。

---

## 附加背景（如需要）

以下是 ChatGPT 可能想知道的补充信息（如果它问起，你可以直接粘贴回去）:

**W2 budget 拦截工作流示例**:
```bash
ctxguard run --budget 150000 --on-full compress -- claude "refactor auth.ts"
```
当 effective_context (input + cache_read + cache_creation) 累计超过 150k tokens 时，自动给 Claude Code 发 `/compact` 命令。

**Show HN 标题候选** (我已经写了 4 个):
- A: Show HN: ctxguard – ulimit for Claude Code's 200k context window
- B: I parsed 7 days of my Claude Code sessions and found 2.1B context tokens
- C: Show HN: ctxguard – a Rust CLI that catches context-window blowups before they cost $1400/wk
- D: Show HN: ctxguard – context-window budgets for Claude Code AND Codex CLI, in 24 ms

**为什么我选 Rust**:
- 812x faster than ccusage on same input
- single 1.1 MB binary, zero npm tree
- notify crate for real-time file watching
- memmap2 for zero-copy JSONL parsing

**我卡住的几个问题**:
1. Username 改 kezhu-20070607 写进了 README 但实际 GitHub 上 repo 还在 kezhu20070607-dotcom（README 8 处 broken link）
2. 3 个 GitHub PAT 在本次 ChatGPT 对话历史里出现过 → 长期泄露 → 必须用户手动撤销
3. v0.2.0 release 缺 Windows binary (GitHub macOS x86_64 runner 队列长)
4. render-demo workflow vhs 失败 (exit 128, cwd 路径异常)

---

请直接开始分析。**不要先问我问题**——基于上面信息直接出 4 周规划。我看到你的输出后会复制回我自己的 Claude Code 执行。