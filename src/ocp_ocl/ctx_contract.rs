#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConformanceProfile {
    P0Core,
    P1Extended,
}

impl ConformanceProfile {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::P0Core => "P0",
            Self::P1Extended => "P1",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CtxFieldType {
    U64,
    U32,
}

impl CtxFieldType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::U64 => "u64",
            Self::U32 => "u32",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CtxSchemaField {
    pub name: &'static str,
    pub ty: CtxFieldType,
}

pub const CORE_SEMANTIC_CTX_FIELDS: [CtxSchemaField; 4] = [
    CtxSchemaField {
        name: "seed",
        ty: CtxFieldType::U64,
    },
    CtxSchemaField {
        name: "tick",
        ty: CtxFieldType::U64,
    },
    CtxSchemaField {
        name: "observer_id",
        ty: CtxFieldType::U64,
    },
    CtxSchemaField {
        name: "policy_id",
        ty: CtxFieldType::U32,
    },
];
