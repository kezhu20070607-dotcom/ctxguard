//! Session-level token summary types.

use tabled::{Table, Tabled};

#[derive(Debug, Clone)]
pub struct TokenSummary {
    pub file: String,
    pub turns: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub model: Option<String>,
    pub first_ts: Option<String>,
    pub last_ts: Option<String>,
}

impl TokenSummary {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
            + self.cache_read_input_tokens
            + self.cache_creation_input_tokens
    }

    /// Effective context-window consumption (most billing APIs treat cache_read as
    /// a fraction of input cost, but the raw count still counts toward the
    /// 200k window).
    pub fn effective_context(&self) -> u64 {
        self.input_tokens + self.cache_read_input_tokens + self.cache_creation_input_tokens
    }

    pub fn duration_minutes(&self) -> Option<u64> {
        let a = self.first_ts.as_deref()?;
        let b = self.last_ts.as_deref()?;
        let pa = chrono::DateTime::parse_from_rfc3339(a).ok()?;
        let pb = chrono::DateTime::parse_from_rfc3339(b).ok()?;
        Some(((pb - pa).num_seconds() / 60).max(0) as u64)
    }

    pub fn print_human(&self) {
        println!("file:        {}", self.file);
        if let Some(m) = &self.model {
            println!("model:       {}", m);
        }
        println!("turns:       {}", self.turns);
        println!(
            "first / last: {:?} / {:?}  ({} min)",
            self.first_ts,
            self.last_ts,
            self.duration_minutes().unwrap_or(0)
        );
        println!("input:       {}", self.input_tokens);
        println!("output:      {}", self.output_tokens);
        println!("cache_read:  {}", self.cache_read_input_tokens);
        println!("cache_write: {}", self.cache_creation_input_tokens);
        println!("---");
        println!("total billed:    {}", self.total_tokens());
        println!("effective ctx:   {}", self.effective_context());
    }

    pub fn print_table(summaries: &[TokenSummary]) {
        if summaries.is_empty() {
            println!("(no sessions in window)");
            return;
        }
        let rows: Vec<Row> = summaries.iter().map(Row::from).collect();
        println!("{}", Table::new(rows));
        let total: u64 = summaries.iter().map(|s| s.total_tokens()).sum();
        let total_ctx: u64 = summaries.iter().map(|s| s.effective_context()).sum();
        println!(
            "\n{} sessions  ·  total billed: {}  ·  effective context: {}",
            summaries.len(),
            total,
            total_ctx
        );
    }
}

#[derive(Tabled)]
struct Row {
    #[tabled(rename = "session")]
    session: String,
    #[tabled(rename = "model")]
    model: String,
    #[tabled(rename = "turns")]
    turns: u64,
    #[tabled(rename = "input")]
    input: String,
    #[tabled(rename = "output")]
    output: String,
    #[tabled(rename = "cache_rd")]
    cache_rd: String,
    #[tabled(rename = "cache_wr")]
    cache_wr: String,
    #[tabled(rename = "ctx_window")]
    ctx_window: String,
}

impl From<&TokenSummary> for Row {
    fn from(s: &TokenSummary) -> Self {
        let short = s
            .file
            .rsplit(|c| c == '/' || c == '\\')
            .next()
            .unwrap_or(&s.file)
            .to_string();
        Row {
            session: short,
            model: s.model.clone().unwrap_or_else(|| "?".into()),
            turns: s.turns,
            input: compact(s.input_tokens),
            output: compact(s.output_tokens),
            cache_rd: compact(s.cache_read_input_tokens),
            cache_wr: compact(s.cache_creation_input_tokens),
            ctx_window: compact(s.effective_context()),
        }
    }
}

fn compact(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
