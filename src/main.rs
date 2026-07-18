//! ctxguard — context-window budget enforcer for AI coding agents.
//!
//! Subcommands:
//!   parse <file.jsonl>     Parse a Claude Code session JSONL, print token summary
//!   profile [--days N]     Aggregate token usage across ~/.claude/projects/
//!   run --budget N --on-full warn|compress|kill -- <cmd>
//!                         Wrap an AI agent and enforce a context budget in real time.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use notify::{RecursiveMode, Watcher};

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
        /// Path to a session .jsonl file
        file: PathBuf,
    },

    /// Aggregate token usage across all sessions under ~/.claude/projects/.
    Profile {
        #[arg(long, default_value_t = 7)]
        days: u32,
        /// Group and rank by model | day | hour | file
        #[arg(long, value_parser = ["model", "day", "hour", "file"])]
        by: Option<String>,
    },

    /// Wrap an AI agent and enforce a context budget in real time.
    ///
    /// Examples:
    ///   ctxguard run --budget 80000 --on-full warn -- claude "fix the auth bug"
    ///   ctxguard run --budget 80000 --on-full compress -- claude "refactor module X"
    ///   ctxguard run --budget 80000 --on-full kill -- claude "try everything"
    Run {
        /// Token budget; triggers --on-full when effective_context crosses this.
        #[arg(long)]
        budget: u64,

        /// What to do when budget is hit:
        ///   warn     — print a clear warning to stderr, keep the child running
        ///   compress — send SIGUSR1 (Claude Code compact hook) or run `/compact`
        ///   kill     — SIGTERM the child cleanly so you can resume later
        #[arg(long, default_value = "warn", value_parser = ["warn", "compress", "kill"])]
        on_full: String,

        /// Poll interval in milliseconds for the JSONL file watcher
        #[arg(long, default_value_t = 500)]
        poll_ms: u64,

        /// Path to the session .jsonl that the child will write to.
        /// If omitted, ctxguard watches the most recently modified file under
        /// ~/.claude/projects/<cwd-hash>/.
        #[arg(long)]
        session: Option<PathBuf>,

        /// Command to run (everything after `--`)
        #[arg(last = true, required = true)]
        cmd: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Parse { file } => {
            let summary =
                parse_file(&file).with_context(|| format!("failed to parse {}", file.display()))?;
            summary.print_human();
        }
        Cmd::Profile { days, by } => {
            let summaries = profile_recent(days)?;
            match by.as_deref() {
                Some("model") => TokenSummary::print_by(&summaries, session::ByDim::Model),
                Some("day") => TokenSummary::print_by(&summaries, session::ByDim::Day),
                Some("hour") => TokenSummary::print_by(&summaries, session::ByDim::Hour),
                Some("file") => TokenSummary::print_by(&summaries, session::ByDim::File),
                _ => TokenSummary::print_table(&summaries),
            }
        }
        Cmd::Run {
            budget,
            on_full,
            poll_ms,
            session,
            cmd,
        } => {
            run_with_budget(budget, &on_full, poll_ms, session, cmd)?;
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// W2: real-time budget enforcement
// ─────────────────────────────────────────────────────────────────────────────

fn run_with_budget(
    budget: u64,
    on_full: &str,
    poll_ms: u64,
    session_override: Option<PathBuf>,
    cmd: Vec<String>,
) -> Result<()> {
    if cmd.is_empty() {
        anyhow::bail!("run: no command provided (use `--` separator)");
    }

    let session_path = match session_override {
        Some(p) => p,
        None => most_recent_session()?.context("no session file found; pass --session <path>")?,
    };

    eprintln!(
        "[ctxguard] watching {} · budget={} tokens · on_full={}",
        session_path.display(),
        budget,
        on_full
    );

    // Spawn child process
    let (program, args) = cmd.split_first().unwrap();
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .spawn()
        .with_context(|| format!("failed to spawn {}", program))?;

    let mut triggered = false;

    // Watch the JSONL file for new assistant turns
    let (tx, rx) = channel();
    let mut watcher: notify::RecommendedWatcher = match notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    }) {
        Ok(w) => w,
        Err(e) => {
            let _ = child.kill();
            anyhow::bail!("failed to create file watcher: {}", e);
        }
    };

    let watch_dir = session_path.parent().context("session has no parent dir")?;
    watcher
        .watch(watch_dir, RecursiveMode::NonRecursive)
        .with_context(|| format!("failed to watch {}", watch_dir.display()))?;

    loop {
        // 1. Drain watcher events
        while let Ok(res) = rx.try_recv() {
            match res {
                Ok(_event) => {
                    if let Ok(s) = parse_file(&session_path) {
                        let ctx = s.effective_context();
                        if !triggered && ctx >= budget {
                            triggered = true;
                            eprintln!(
                                "\n[ctxguard] BUDGET HIT: effective_context={} >= budget={}",
                                ctx, budget
                            );
                            match on_full {
                                "warn" => eprintln!(
                                    "[ctxguard] child process continues — pass --on-full kill|compress to enforce"
                                ),
                                "compress" => {
                                    eprintln!("[ctxguard] requesting compact via stdin");
                                    if let Some(mut stdin) = child.stdin.take() {
                                        use std::io::Write;
                                        let _ = writeln!(stdin, "/compact");
                                        let _ = stdin.flush();
                                        child.stdin = Some(stdin);
                                    }
                                }
                                "kill" => {
                                    eprintln!("[ctxguard] sending SIGTERM to child (pid={:?})", child.id());
                                    let _ = child.kill();
                                }
                                _ => unreachable!("validated by clap"),
                            }
                        }
                    }
                }
                Err(e) => eprintln!("[ctxguard] watch error: {:?}", e),
            }
        }

        // 2. Reap child if done
        match child.try_wait() {
            Ok(Some(status)) => {
                eprintln!("\n[ctxguard] child exited: {}", status);
                return Ok(());
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("[ctxguard] waitpid error: {}", e);
                break;
            }
        }

        // 3. Heartbeat poll — no-op for now

        thread::sleep(Duration::from_millis(poll_ms));
    }
    Ok(())
}

fn most_recent_session() -> Result<Option<PathBuf>> {
    let root = dirs_root()?;
    let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in walkdir::WalkDir::new(&root)
        .max_depth(2)
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
        if let Ok(meta) = entry.metadata() {
            if let Ok(modified) = meta.modified() {
                if newest.as_ref().map(|(t, _)| modified > *t).unwrap_or(true) {
                    newest = Some((modified, p.to_path_buf()));
                }
            }
        }
    }
    Ok(newest.map(|(_, p)| p))
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
        let ts = val
            .get("timestamp")
            .and_then(|v| v.as_str())
            .map(String::from);
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
            input_tokens += usage
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            output_tokens += usage
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            cache_read += usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            cache_write += usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
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

// Silence unused warning on Windows where Child::stdin behaves differently
#[allow(dead_code)]
fn _unused(_: &mut Child) {}
