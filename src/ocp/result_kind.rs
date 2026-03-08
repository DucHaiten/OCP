use crate::ocp::ReasonCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultKind {
    Ok,
    Degraded,
    Insufficient,
    Deferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Result4<T> {
    pub kind: ResultKind,
    pub payload: Option<T>,
    pub reason: Option<ReasonCode>,
    pub origin_id: Option<u64>,
}

impl<T> Result4<T> {
    pub fn ok(payload: T) -> Self {
        Self {
            kind: ResultKind::Ok,
            payload: Some(payload),
            reason: None,
            origin_id: None,
        }
    }

    pub fn degraded(payload: T, reason: ReasonCode) -> Self {
        Self {
            kind: ResultKind::Degraded,
            payload: Some(payload),
            reason: Some(reason),
            origin_id: None,
        }
    }

    pub fn insufficient(reason: ReasonCode) -> Self {
        Self {
            kind: ResultKind::Insufficient,
            payload: None,
            reason: Some(reason),
            origin_id: None,
        }
    }

    pub fn deferred(reason: ReasonCode) -> Self {
        Self {
            kind: ResultKind::Deferred,
            payload: None,
            reason: Some(reason),
            origin_id: None,
        }
    }

    pub fn with_origin_id(mut self, origin_id: u64) -> Self {
        self.origin_id = Some(origin_id);
        self
    }
}
