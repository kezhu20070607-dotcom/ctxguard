//! Token counting facade.
//!
//! Right now we trust the `usage` block that Claude Code writes into the JSONL,
//! which already accounts for tokenizer choice. For W2/W3 we may want a local
//! tokenizer fallback (tiktoken-rs) to estimate when usage is missing or when
//! the user pipes stdout that has no JSONL side-channel.

#[allow(dead_code)]
pub fn humanize_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
