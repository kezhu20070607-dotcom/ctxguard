//! ctxguard — context-window budget enforcer for AI coding agents.
//!
//! Subcommands:
//!   parse <file.jsonl>     Parse a Claude Code session JSONL, print token summary
//!   profile [--days N]     Aggregate token usage across ~/.claude/projects/
//!   run --budget N -- <cmd>   Wrap an AI agent, enforce context budget in real time (W2)

use std::path::PathBuf;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod session;
mod token;

use session::TokenSummary;

#[derive(Parser, Debug)]
#[command(name = "ctxguard", version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Parse a single Claude Code session JSONL and print its token summary.
    Parse {
        /// Path to a session .jsonl file (e.g. ~/.claude/projects/<proj>/<sessionId>.jsonl)
        file: PathBuf,
    },

    /// Aggregate token usage across all sessions under ~/.claude/projects/.
    Profile {
        /// Only include sessions modified within the last N days (default: 7)
        #[arg(long, default_value_t = 7)]
        days: u32,
    },

    /// Wrap an AI agent and enforce a context budget in real time.
    /// TODO (W2): implement real-time stdin/stdout monitoring + signal trigger.
    Run {
        /// Token budget; warn or compress when cumulative usage crosses this.
        #[arg(long)]
        budget: u64,
        /// What to do when budget is hit: warn | compress | kill
        #[arg(long, default_value = "warn")]
        on_full: String,
        /// Command to run (everything after `--`)
        #[arg(last = true, required = true)]
        cmd: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Parse { file } => {
            let summary = parse_file(&file)
                .with_context(|| format!("failed to parse {}", file.display()))?;
            summary.print_human();
        }
        Cmd::Profile { days } => {
            let summaries = profile_recent(days)?;
            TokenSummary::print_table(&summaries);
        }
        Cmd::Run { budget, on_full, cmd } => {
            anyhow::bail!(
                "ctxguard run not implemented yet (W2).\n\
                 requested: budget={budget}, on_full={on_full}, cmd={cmd:?}\n\
                 TODO: spawn child process, tail session.jsonl in real time, \
                 signal on threshold."
            );
        }
    }
    Ok(())
}

fn parse_file(path: &PathBuf) -> Result<TokenSummary> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let f = File::open(path)?;
    let reader = BufReader::with_capacity(64 * 1024, f);

    let mut turns = 0u64;
    let mut input_tokens = 0u64;
    let mut output_tokens = 0u64;
    let mut cache_read = 0u64;
    let mut cache_write = 0u64;
    let mut first_ts: Option<String> = None;
    let mut last_ts: Option<String> = None;
    let mut model: Option<String> = None;

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let val: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let ts = val.get("timestamp").and_then(|v| v.as_str()).map(String::from);
        if let Some(t) = &ts {
            if first_ts.is_none() {
                first_ts = Some(t.clone());
            }
            last_ts = Some(t.clone());
        }

        if val.get("type").and_then(|v| v.as_str()) != Some("assistant") {
            continue;
        }

        let msg = match val.get("message") {
            Some(m) => m,
            None => continue,
        };

        if model.is_none() {
            if let Some(m) = msg.get("model").and_then(|v| v.as_str()) {
                model = Some(m.to_string());
            }
        }

        if let Some(usage) = msg.get("usage") {
            turns += 1;
            input_tokens += usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            output_tokens += usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            cache_read += usage.get("cache_read_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            cache_write += usage.get("cache_creation_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
        }
    }

    Ok(TokenSummary {
        file: path.to_string_lossy().to_string(),
        turns,
        input_tokens,
        output_tokens,
        cache_read_input_tokens: cache_read,
        cache_creation_input_tokens: cache_write,
        model,
        first_ts,
        last_ts,
    })
}

fn profile_recent(days: u32) -> Result<Vec<TokenSummary>> {
    let claude_root = dirs_root()?;
    let mut out = Vec::new();

    for entry in walkdir::WalkDir::new(&claude_root)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("jsonl") {
            continue;
        }
        if p.to_string_lossy().contains("subagents") {
            continue;
        }
        let modified = entry.metadata().ok().and_then(|m| m.modified().ok());
        if let Some(mtime) = modified {
            if let Ok(elapsed) = mtime.elapsed() {
                if elapsed.as_secs() > (days as u64) * 86400 {
                    continue;
                }
            }
        }
        if let Ok(s) = parse_file(&p.to_path_buf()) {
            out.push(s);
        }
    }
    Ok(out)
}

fn dirs_root() -> Result<PathBuf> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .context("neither USERPROFILE nor HOME is set")?;
    let mut p = PathBuf::from(home);
    p.push(".claude");
    p.push("projects");
    Ok(p)
}
