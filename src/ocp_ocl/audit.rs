use crate::ocp_ocl::{ReasonCode, ResultKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceEvent {
    ObserveStart {
        key: String,
        tier: String,
        ctx: String,
        budget: u32,
    },
    ObserveEnd {
        key: String,
        kind: ResultKind,
        reason: Option<ReasonCode>,
        origin_id: u64,
    },
    UiObserve {
        key: String,
        event_count: u32,
        truncated: bool,
        detail: String,
        kind: ResultKind,
        reason: Option<ReasonCode>,
    },
    GameObserve {
        key: String,
        stream: String,
        value_count: u32,
        tick: i64,
        kind: ResultKind,
        reason: Option<ReasonCode>,
    },
    ShadowRun {
        key: String,
        branch_count: u32,
        truncated: bool,
        detail: String,
        kind: ResultKind,
        reason: Option<ReasonCode>,
    },
    ShadowCompare {
        key: String,
        diff_count: u32,
        report_bytes: u32,
        truncated: bool,
        kind: ResultKind,
        reason: Option<ReasonCode>,
    },
    MatchArmSelected {
        arm: ResultKind,
    },
    CommitAttempt {
        origin_id: Option<u64>,
        kind: Option<ResultKind>,
    },
    CommitResult {
        allowed: bool,
        reason: Option<ReasonCode>,
    },
    UiCommit {
        key: String,
        cmd_count: u32,
        present: bool,
        kind: ResultKind,
        reason: Option<ReasonCode>,
    },
    GameCommit {
        key: String,
        delta_bytes: u32,
        idempotency_hash: String,
        kind: ResultKind,
        reason: Option<ReasonCode>,
    },
    ConditionCheck {
        value: bool,
    },
    LoopIter {
        loop_kind: String,
        iter_index: u32,
    },
    ProgramEnd {
        steps: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TraceLog {
    pub events: Vec<TraceEvent>,
}

impl TraceLog {
    pub fn push(&mut self, event: TraceEvent) {
        self.events.push(event);
    }

    pub fn signature_hex(&self) -> String {
        let mut hash = 0xcbf29ce484222325u64;
        for ev in &self.events {
            let line = serialize_event(ev);
            for b in line.as_bytes() {
                hash ^= u64::from(*b);
                hash = hash.wrapping_mul(0x100000001b3);
            }
            hash ^= u64::from(b'\n');
            hash = hash.wrapping_mul(0x100000001b3);
        }
        format!("{hash:016x}")
    }
}

fn serialize_event(ev: &TraceEvent) -> String {
    match ev {
        TraceEvent::ObserveStart {
            key,
            tier,
            ctx,
            budget,
        } => format!("ObserveStart|{key}|{tier}|{ctx}|{budget}"),
        TraceEvent::ObserveEnd {
            key,
            kind,
            reason,
            origin_id,
        } => format!(
            "ObserveEnd|{key}|{:?}|{}|{origin_id}",
            kind,
            reason.map(|r| r.as_str()).unwrap_or("-")
        ),
        TraceEvent::UiObserve {
            key,
            event_count,
            truncated,
            detail,
            kind,
            reason,
        } => format!(
            "UiObserve|{key}|{event_count}|{}|{:?}|{}|{detail}",
            if *truncated { "1" } else { "0" },
            kind,
            reason.map(|r| r.as_str()).unwrap_or("-"),
        ),
        TraceEvent::GameObserve {
            key,
            stream,
            value_count,
            tick,
            kind,
            reason,
        } => format!(
            "GameObserve|{key}|{stream}|{value_count}|{tick}|{:?}|{}",
            kind,
            reason.map(|r| r.as_str()).unwrap_or("-"),
        ),
        TraceEvent::ShadowRun {
            key,
            branch_count,
            truncated,
            detail,
            kind,
            reason,
        } => format!(
            "ShadowRun|{key}|{branch_count}|{}|{detail}|{:?}|{}",
            if *truncated { "1" } else { "0" },
            kind,
            reason.map(|r| r.as_str()).unwrap_or("-"),
        ),
        TraceEvent::ShadowCompare {
            key,
            diff_count,
            report_bytes,
            truncated,
            kind,
            reason,
        } => format!(
            "ShadowCompare|{key}|{diff_count}|{report_bytes}|{}|{:?}|{}",
            if *truncated { "1" } else { "0" },
            kind,
            reason.map(|r| r.as_str()).unwrap_or("-"),
        ),
        TraceEvent::MatchArmSelected { arm } => format!("MatchArmSelected|{:?}", arm),
        TraceEvent::CommitAttempt { origin_id, kind } => format!(
            "CommitAttempt|{}|{}",
            origin_id
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            kind.map(|k| format!("{k:?}"))
                .unwrap_or_else(|| "-".to_string())
        ),
        TraceEvent::CommitResult { allowed, reason } => format!(
            "CommitResult|{}|{}",
            if *allowed { "allow" } else { "deny" },
            reason.map(|r| r.as_str()).unwrap_or("-")
        ),
        TraceEvent::UiCommit {
            key,
            cmd_count,
            present,
            kind,
            reason,
        } => format!(
            "UiCommit|{key}|{cmd_count}|{}|{:?}|{}",
            if *present { "1" } else { "0" },
            kind,
            reason.map(|r| r.as_str()).unwrap_or("-")
        ),
        TraceEvent::GameCommit {
            key,
            delta_bytes,
            idempotency_hash,
            kind,
            reason,
        } => format!(
            "GameCommit|{key}|{delta_bytes}|{idempotency_hash}|{:?}|{}",
            kind,
            reason.map(|r| r.as_str()).unwrap_or("-"),
        ),
        TraceEvent::ConditionCheck { value } => format!("ConditionCheck|{value}"),
        TraceEvent::LoopIter {
            loop_kind,
            iter_index,
        } => format!("LoopIter|{loop_kind}|{iter_index}"),
        TraceEvent::ProgramEnd { steps } => format!("ProgramEnd|{steps}"),
    }
}
