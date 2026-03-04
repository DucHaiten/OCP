use std::collections::BTreeMap;

use crate::ocp_ocl::Result4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
    List(Vec<Value>),
    Map(BTreeMap<String, Value>),
    Budget(u32),
    Ctx(String),
    Payload(BTreeMap<String, String>),
    Result4(Box<Result4<Value>>),
    Unit,
    Unknown,
}

impl Value {
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::String(v) => Some(v.clone()),
            _ => None,
        }
    }
}
