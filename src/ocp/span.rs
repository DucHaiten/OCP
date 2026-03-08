#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub file_id: u32,
    pub start: u32,
    pub end: u32,
    pub line: u32,
    pub column: u32,
}

impl Span {
    pub const fn new(file_id: u32, start: u32, end: u32, line: u32, column: u32) -> Self {
        Self {
            file_id,
            start,
            end,
            line,
            column,
        }
    }
}
