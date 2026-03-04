use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
    String,
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Record(BTreeMap<String, Type>),
    Budget,
    Ctx,
    Payload,
    Result4(Box<Type>),
    Unit,
    Unknown,
}

impl Type {
    pub fn result4_payload() -> Self {
        Self::Result4(Box::new(Self::Payload))
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Bool => "bool",
            Self::String => "string",
            Self::List(_) => "list",
            Self::Map(_, _) => "map",
            Self::Record(_) => "record",
            Self::Budget => "budget",
            Self::Ctx => "ctx",
            Self::Payload => "payload",
            Self::Result4(_) => "result4",
            Self::Unit => "unit",
            Self::Unknown => "unknown",
        }
    }
}
