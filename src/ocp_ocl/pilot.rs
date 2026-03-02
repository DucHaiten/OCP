use crate::ocp_ocl::{run_fixture_source, ExecConfig, FixtureRunnerError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PilotTurnStatus {
    Ok,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PilotTurnResult {
    pub turn_index: usize,
    pub status: PilotTurnStatus,
    pub signature: String,
    pub steps: u32,
    pub commit_count: usize,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PilotSummary {
    pub turns: Vec<PilotTurnResult>,
    pub aggregate_signature: String,
    pub ok_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PilotReplayResult {
    pub first: PilotSummary,
    pub second: PilotSummary,
    pub deterministic: bool,
}

pub fn run_pilot_sequence(sources: &[&str], config: ExecConfig) -> PilotSummary {
    let mut turns = Vec::with_capacity(sources.len());
    for (i, src) in sources.iter().enumerate() {
        turns.push(run_one_turn(i, src, config));
    }
    summarize(turns)
}

pub fn replay_pilot_sequence(sources: &[&str], config: ExecConfig) -> PilotReplayResult {
    let first = run_pilot_sequence(sources, config);
    let second = run_pilot_sequence(sources, config);
    let deterministic = first.aggregate_signature == second.aggregate_signature;
    PilotReplayResult {
        first,
        second,
        deterministic,
    }
}

pub fn build_closeout_report(summary: &PilotSummary, replay: &PilotReplayResult) -> String {
    let mut out = String::new();
    out.push_str("# OCP-OCL v0.1 Pilot Closeout\n\n");
    out.push_str("## Summary\n");
    out.push_str(&format!("- turns: {}\n", summary.turns.len()));
    out.push_str(&format!("- ok_count: {}\n", summary.ok_count));
    out.push_str(&format!("- error_count: {}\n", summary.error_count));
    out.push_str(&format!(
        "- aggregate_signature: `{}`\n",
        summary.aggregate_signature
    ));
    out.push_str(&format!(
        "- replay_deterministic: `{}`\n\n",
        replay.deterministic
    ));

    out.push_str("## Turn Details\n");
    for turn in &summary.turns {
        out.push_str(&format!(
            "- turn {}: status={:?}, steps={}, commits={}, signature=`{}`",
            turn.turn_index, turn.status, turn.steps, turn.commit_count, turn.signature
        ));
        if let Some(code) = &turn.error_code {
            out.push_str(&format!(", error_code=`{}`", code));
        }
        out.push('\n');
    }
    out.push('\n');

    out.push_str("## Replay Check\n");
    out.push_str(&format!(
        "- first_signature: `{}`\n",
        replay.first.aggregate_signature
    ));
    out.push_str(&format!(
        "- second_signature: `{}`\n",
        replay.second.aggregate_signature
    ));
    out.push_str(&format!("- deterministic: `{}`\n", replay.deterministic));

    out
}

fn run_one_turn(turn_index: usize, source: &str, config: ExecConfig) -> PilotTurnResult {
    match run_fixture_source(source, (turn_index + 1) as u32, config) {
        Ok(out) => PilotTurnResult {
            turn_index,
            status: PilotTurnStatus::Ok,
            signature: out.signature,
            steps: out.steps,
            commit_count: out.commits.len(),
            error_code: None,
        },
        Err(err) => PilotTurnResult {
            turn_index,
            status: PilotTurnStatus::Error,
            signature: error_signature(turn_index, &err),
            steps: 0,
            commit_count: 0,
            error_code: to_error_code(&err),
        },
    }
}

fn summarize(turns: Vec<PilotTurnResult>) -> PilotSummary {
    let ok_count = turns
        .iter()
        .filter(|t| matches!(t.status, PilotTurnStatus::Ok))
        .count();
    let error_count = turns.len().saturating_sub(ok_count);
    let aggregate_signature = aggregate_turn_signatures(&turns);
    PilotSummary {
        turns,
        aggregate_signature,
        ok_count,
        error_count,
    }
}

fn to_error_code(err: &FixtureRunnerError) -> Option<String> {
    match err {
        FixtureRunnerError::Io(_) => Some("IO".to_string()),
        FixtureRunnerError::Diag(d) => Some(d.code.as_str().to_string()),
    }
}

fn error_signature(turn_index: usize, err: &FixtureRunnerError) -> String {
    let line = match err {
        FixtureRunnerError::Io(msg) => format!("turn:{turn_index}|io:{msg}"),
        FixtureRunnerError::Diag(d) => format!(
            "turn:{turn_index}|diag:{}|msg:{}|span:{}:{}-{}",
            d.code.as_str(),
            d.message,
            d.span.file_id,
            d.span.start,
            d.span.end
        ),
    };
    fnv1a64_hex(&line)
}

fn aggregate_turn_signatures(turns: &[PilotTurnResult]) -> String {
    let mut acc = String::new();
    for t in turns {
        acc.push_str(&format!(
            "{}|{:?}|{}|{}|{}|{}\n",
            t.turn_index,
            t.status,
            t.signature,
            t.steps,
            t.commit_count,
            t.error_code.clone().unwrap_or_else(|| "-".to_string())
        ));
    }
    fnv1a64_hex(&acc)
}

fn fnv1a64_hex(input: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for b in input.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}
