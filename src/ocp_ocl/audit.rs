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
        TraceEvent::ConditionCheck { value } => format!("ConditionCheck|{value}"),
        TraceEvent::LoopIter {
            loop_kind,
            iter_index,
        } => format!("LoopIter|{loop_kind}|{iter_index}"),
        TraceEvent::ProgramEnd { steps } => format!("ProgramEnd|{steps}"),
    }
}
